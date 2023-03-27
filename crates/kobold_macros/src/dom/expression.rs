use std::fmt::{self, Debug};

use proc_macro::{Group, Ident, Span, TokenStream, TokenTree};

use crate::dom::Node;
use crate::parse::IdentExt;
use crate::tokenize::prelude::*;

pub struct Expression {
    pub stream: TokenStream,
    pub span: Span,
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

        Expression { stream, span }
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
                "for" | "override" => Some(Ident::new_raw(ident, span)),
                "ref" => {
                    deref = true;

                    Some(Ident::new_raw(ident, span))
                }
                "static" => {
                    // deref = true;
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
                };
            }
        }

        Expression {
            stream: stream.collect(),
            span: group.span(),
        }
    }
}

impl From<&str> for Expression {
    fn from(code: &str) -> Self {
        Expression {
            stream: code.tokenize(),
            span: Span::call_site(),
        }
    }
}
