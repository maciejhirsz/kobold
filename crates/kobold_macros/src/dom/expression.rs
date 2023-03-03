use std::fmt::{self, Debug};

use proc_macro::{Group, Span, TokenStream, TokenTree};

use crate::dom::Node;
use crate::parse::IteratorExt;
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

        let stream = if stream.allow_consume("static").is_some() {
            call("::kobold::util::Static", stream).tokenize()
        } else {
            stream.collect()
        };

        Expression {
            stream,
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
