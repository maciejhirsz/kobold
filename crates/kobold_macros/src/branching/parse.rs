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

enum Token {
    Html,
    If,
}

impl Token {
    fn from_str(ident: &str) -> Option<Token> {
        match ident {
            "html" => Some(Token::Html),
            "if" => Some(Token::If),
            _ => None,
        }
    }
}

type Branches<'a> = Option<&'a Rc<Cell<u8>>>;

struct ShallowParser<'a> {
    stream: &'a mut ParseStream,
}

impl Iterator for ShallowParser<'_> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stream.allow('{') {
            return None;
        }

        self.stream.next()
    }
}

fn parse_code(stream: &mut ParseStream, scope: Branches) -> Result<Vec<Code>, ParseError> {
    let mut code = CodeBuilder::default();

    while let Some(tt) = stream.next() {
        match tt {
            TokenTree::Ident(ref ident) => {
                match ident.with_str(Token::from_str) {
                    Some(Token::Html) => {
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

                                code.push(Code::Html(Html::new(
                                    html.into_iter().collect(),
                                    branches,
                                )));
                            }
                            Err(tokens) => code.extend(tokens),
                        }

                        continue;
                    }
                    Some(Token::If) => {
                        let scope = Rc::new(Cell::new(0));
                        let scope = Some(&scope);

                        code.collect(tt);
                        code.extend(ShallowParser { stream });

                        if let TokenTree::Group(group) = stream.expect('{')? {
                            code.push(Code::Nested(Nested::parse(group, scope)?));
                        }

                        loop {
                            if let Some(tt) = stream.allow_consume("else") {
                                code.collect(tt);

                                let else_if = stream.allow_consume("if");
                                let stop = else_if.is_none();

                                if let Some(tt) = else_if {
                                    code.collect(tt);
                                    code.extend(ShallowParser { stream });
                                }

                                if let TokenTree::Group(group) = stream.expect('{')? {
                                    code.push(Code::Nested(Nested::parse(group, scope)?));
                                }

                                if stop {
                                    break;
                                }
                            }
                        }

                        continue;
                    }
                    None => (),
                }

                code.collect(tt);
            }
            TokenTree::Group(group) => {
                code.push(Code::Nested(Nested::parse(group, scope)?));
            }
            tt => {
                code.collect(tt);
            }
        }
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
