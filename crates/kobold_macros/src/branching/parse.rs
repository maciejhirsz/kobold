// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::Cell;
use std::rc::Rc;

use arrayvec::ArrayVec;
use tokens::{Group, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::TokenStreamExt;

use super::ast::{Code, Nested, Scope, Scoped};

#[derive(Default)]
struct CodeBuilder {
    code: Vec<Code>,
    segment: Option<TokenStream>,
}

impl CodeBuilder {
    pub fn push(&mut self, code: Code) {
        self.segment();
        self.code.push(code);
    }

    pub fn collect(&mut self, tt: TokenTree) -> usize {
        self.segment
            .get_or_insert_with(TokenStream::new)
            .extend([tt]);

        self.code.len()
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = TokenTree>) {
        let segment = self.segment.get_or_insert_with(TokenStream::new);

        segment.extend(iter);
    }

    pub fn finish(mut self) -> Vec<Code> {
        self.segment();
        self.code
    }

    fn segment(&mut self) {
        if let Some(segment) = self.segment.take() {
            self.code.push(Code::Segment(segment))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Token {
    /// The `html` identifier
    View,
    /// The `if` keyword
    If,
    /// The `else` keyword
    Else,
    /// The `match` keyword,
    Match,
    /// The `{ ... }` block
    Block,
    /// Any `Group` other than `{ ... }` block
    Group,
}

impl Token {
    fn from_str(ident: &str) -> Option<Token> {
        match ident {
            "view" => Some(Token::View),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "match" => Some(Token::Match),
            _ => None,
        }
    }

    fn get(tt: &TokenTree) -> Option<Token> {
        match tt {
            TokenTree::Ident(ident) => ident.with_str(Token::from_str),
            tt if tt.is('{') => Some(Token::Block),
            TokenTree::Group(_) => Some(Token::Group),
            _ => None,
        }
    }
}

type Branches<'a> = Option<&'a Rc<Cell<u8>>>;

fn parse_code(stream: &mut ParseStream, scope: Branches) -> Result<Vec<Code>, ParseError> {
    let mut code = CodeBuilder::default();

    enum Mode {
        /// Next token must match state
        Eager,

        /// Allow other tokens until a match
        Lazy,
    }

    struct State(Token, Mode, Rc<Cell<u8>>);

    impl State {
        fn scope(self) -> Rc<Cell<u8>> {
            self.2
        }
    }

    let mut states = Vec::<State>::new();

    while let Some(tt) = stream.next() {
        let token = match Token::get(&tt) {
            Some(token) => token,
            None => {
                code.collect(tt);
                continue;
            }
        };

        let state = loop {
            match states.last() {
                Some(State(expect, mode, _)) => {
                    if *expect == token {
                        // Match! Get owned state
                        break states.pop();
                    } else {
                        match mode {
                            // No match, discard state
                            Mode::Eager => {
                                let _ = states.pop();
                                continue;
                            }
                            // No match, retain state
                            Mode::Lazy => break None,
                        }
                    }
                }
                _ => break None,
            }
        };

        match token {
            Token::View => {
                let mut maybe_html = ArrayVec::<_, 3>::new();

                maybe_html.push(tt);

                if let Some(tt) = stream.allow_consume('!') {
                    maybe_html.push(tt);

                    if let Some(tt) = stream.next_if(|tt| matches!(tt, TokenTree::Group(_))) {
                        maybe_html.push(tt);
                    }
                }

                match maybe_html.into_inner() {
                    Ok(html) => {
                        let branches = scope.cloned();
                        let span = html[0].span();

                        code.push(Code::Scoped(Scoped::new(
                            html.into_iter().collect(),
                            span,
                            branches,
                        )));
                    }
                    Err(tokens) => code.extend(tokens),
                }

                continue;
            }
            Token::If => {
                let scope = state.map(State::scope).unwrap_or_default();

                states.push(State(Token::Else, Mode::Eager, scope.clone()));
                states.push(State(Token::Block, Mode::Lazy, scope));
            }
            Token::Else => {
                if let Some(scope) = state.map(State::scope) {
                    states.push(State(Token::If, Mode::Eager, scope.clone()));
                    states.push(State(Token::Block, Mode::Eager, scope));
                }
            }
            Token::Match => {
                states.push(State(Token::Block, Mode::Lazy, Rc::default()));
            }
            Token::Block | Token::Group => {
                let state_scope = state.map(State::scope);
                let scope = state_scope.as_ref().or(scope);

                match tt {
                    TokenTree::Group(group) => {
                        code.push(Code::Nested(Nested::parse(group, scope)?))
                    }
                    _ => panic!("Token not matching TokenTree, this is a bug, please report it"),
                }

                continue;
            }
        }

        code.collect(tt);
    }

    Ok(code.finish())
}

impl Parse for Scope {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        Ok(Scope {
            code: parse_code(stream, None)?,
        })
    }
}

impl Nested {
    fn parse(group: Group, scope: Branches) -> Result<Self, ParseError> {
        let code = parse_code(&mut group.stream().parse_stream(), scope)?;

        Ok(Nested {
            delimiter: group.delimiter(),
            code,
            span: group.span(),
        })
    }
}
