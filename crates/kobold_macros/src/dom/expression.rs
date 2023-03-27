// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{self, Debug};

use proc_macro::{Group, Ident, Span, TokenStream, TokenTree};

use crate::dom::Node;
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

impl From<TokenTree> for Expression {
    fn from(tt: TokenTree) -> Self {
        if let TokenTree::Group(group) = tt {
            return Expression::from(group);
        }

        let span = tt.span();
        let stream = tt.tokenize();

        Expression { stream, span, is_static: false }
    }
}

impl From<Group> for Expression {
    fn from(group: Group) -> Self {
        let mut stream = group.stream().parse_stream();

        if let Some(TokenTree::Ident(ident)) = stream.peek() {
            let span = ident.span();
            let mut is_static = false;
            let mut deref = false;

            let keyword = ident.with_str(|ident| match ident {
                "for" | "use" => Some(Ident::new_raw(ident, span)),
                "ref" => {
                    deref = true;

                    Some(Ident::new_raw(ident, span))
                }
                "static" => {
                    is_static = true;

                    Some(Ident::new_raw(ident, span))
                }
                _ => None,
            });

            if let Some(keyword) = keyword {
                stream.next();

                return Expression {
                    stream: call(
                        ("::kobold::keywords::", keyword),
                        (deref.then_some('&'), stream),
                    ),
                    span: group.span(),
                    is_static,
                };
            }
        }

        Expression {
            stream: stream.collect(),
            span: group.span(),
            is_static: false,
        }
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
