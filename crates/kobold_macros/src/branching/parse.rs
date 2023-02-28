use std::cell::Cell;
use std::rc::Rc;

use arrayvec::ArrayVec;
use proc_macro::{Group, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::TokenStreamExt;

use super::ast::{Scope, Code, Html, Nested};

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
}

impl Token {
    fn from_str(ident: &str) -> Option<Token> {
        match ident {
            "html" => Some(Token::Html),
            _ => None,
        }
    }
}

fn parse_code(
    stream: &mut ParseStream,
    branches: &Rc<Cell<usize>>,
) -> Result<Vec<Code>, ParseError> {
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
                                code.push(Code::Html(Html {
                                    tokens: html.into_iter().collect(),
                                    branch: branches.get(),
                                    branches: branches.clone(),
                                }));
                                branches.set(branches.get() + 1);
                            }
                            Err(tokens) => code.extend(tokens),
                        }

                        continue;
                    }
                    None => (),
                }

                code.collect(tt);
            }
            TokenTree::Group(group) => {
                code.push(Code::Nested(Nested::parse(group, &branches)?));
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
        let branches = Rc::new(Cell::new(0));

        Ok(Scope {
            code: parse_code(stream, &branches)?,
            branches,
        })
    }
}

impl Nested {
    fn parse(group: Group, branches: &Rc<Cell<usize>>) -> Result<Self, ParseError> {
        let code = parse_code(&mut group.stream().parse_stream(), branches)?;

        Ok(Nested {
            delimiter: group.delimiter(),
            code,
            span: group.span(),
        })
    }
}
