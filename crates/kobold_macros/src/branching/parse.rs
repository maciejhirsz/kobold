use std::cell::Cell;
use std::rc::Rc;

use arrayvec::ArrayVec;
use proc_macro::{Group, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::TokenStreamExt;

use super::ast::{Code, Html, Nested, Scope};

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
    Html,
    /// The `if` keyword
    If,
    /// The `else` keyword
    Else,
    /// The `{ ... }` block
    Block,
    /// Any `Group` other than `{ ... }` block
    Group,
}

impl Token {
    fn from_str(ident: &str) -> Option<Token> {
        match ident {
            "html" => Some(Token::Html),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
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

    let mut states = Vec::<State>::new();

    while let Some(tt) = stream.next() {
        let token = match Token::get(&tt) {
            Some(token) => token,
            None => {
                code.collect(tt);
                continue;
            }
        };
        let state = match states.last() {
            Some(State(expect, mode, _)) => {
                if *expect == token {
                    // Match! Get owned state
                    states.pop()
                } else {
                    match mode {
                        // No match, discard state
                        Mode::Eager => {
                            let _ = states.pop();
                        }
                        // No match, retain state
                        Mode::Lazy => (),
                    }
                    None
                }
            }
            _ => None,
        };

        match token {
            Token::Html => {
                let mut maybe_html = ArrayVec::<_, 3>::new();

                maybe_html.push(tt);
                maybe_html.extend(
                    ['!', '{']
                        .into_iter()
                        .map_while(|p| stream.allow_consume(p)),
                );

                match maybe_html.into_inner() {
                    Ok(html) => {
                        let branches = scope.map(Clone::clone);

                        code.push(Code::Html(Html::new(html.into_iter().collect(), branches)));
                    }
                    Err(tokens) => code.extend(tokens),
                }

                continue;
            }
            Token::If => {
                let scope = Rc::new(Cell::new(0));

                states.push(State(Token::Else, Mode::Eager, scope.clone()));
                states.push(State(Token::Block, Mode::Lazy, scope));

                code.collect(tt);

                continue;
            }
            Token::Else => {
                if let Some(State(_, _, scope)) = state {
                    code.collect(tt);

                    if let Some(tt) = stream.allow_consume("if") {
                        code.collect(tt);

                        states.push(State(Token::Else, Mode::Eager, scope.clone()));
                        states.push(State(Token::Block, Mode::Lazy, scope));
                    } else {
                        states.push(State(Token::Block, Mode::Eager, scope));
                    };

                    continue;
                }
            }
            Token::Block | Token::Group => {
                let scope = state.as_ref().map(|State(_, _, ref scope)| scope).or(scope);

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
