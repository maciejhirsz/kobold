// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{self, Debug};

use tokens::{Group, Ident, Span, TokenStream, TokenTree};

use crate::dom::{IteratorExt, Lit, Node, ParseError};
use crate::parse::IdentExt;
use crate::tokenize::prelude::*;

pub struct Expression {
    pub stream: TokenStream,
    pub span: Span,
    pub is_static: bool,
}

impl Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.stream, f)
    }
}

impl From<Expression> for Node {
    fn from(expr: Expression) -> Node {
        Node::Expression(expr)
    }
}

impl TryFrom<TokenTree> for Expression {
    type Error = ParseError;

    fn try_from(tt: TokenTree) -> Result<Self, ParseError> {
        if let TokenTree::Group(group) = tt {
            return Ok(Expression::try_from(group)?);
        }

        let span = tt.span();
        let stream = tt.tokenize();

        Ok(Expression {
            stream,
            span,
            is_static: false,
        })
    }
}

impl TryFrom<Group> for Expression {
    type Error = ParseError;

    fn try_from(group: Group) -> Result<Self, ParseError> {
        let mut stream = group.stream().parse_stream();

        if let Some(TokenTree::Ident(ident)) = stream.peek() {
            let span = ident.span();
            let mut is_static = false;
            let mut deref = false;
            let mut invoke = None;

            let keyword = ident.with_str(|ident| match ident {
                "for" => Some("for"),
                "use" => Some("use"),
                "ref" => {
                    deref = true;

                    Some("ref")
                }
                "static" => {
                    is_static = true;

                    Some("static")
                }
                "do" => {
                    invoke = Some('!'.tokenize());

                    Some("do")
                }
                _ => None,
            });

            if let Some(mut keyword) = keyword {
                stream.next();

                if keyword == "for" {
                    if let Some(_) = stream.allow_consume('<') {
                        let n = stream.expect(Lit)?;
                        let close = stream.expect('>')?;

                        keyword = "for_bounded";
                        invoke = Some(("::<_, ", n, close).tokenize())
                    }
                }
                let keyword = Ident::new_raw(keyword, span);

                return Ok(Expression {
                    stream: call(
                        ("::kobold::keywords::", keyword, invoke),
                        (deref.then_some('&'), stream),
                    ),
                    span: group.span(),
                    is_static,
                });
            }
        }

        Ok(Expression {
            stream: stream.collect(),
            span: group.span(),
            is_static: false,
        })
    }
}

impl From<&str> for Expression {
    fn from(code: &str) -> Self {
        Expression {
            stream: code.tokenize(),
            span: Span::call_site(),
            is_static: false,
        }
    }
}
