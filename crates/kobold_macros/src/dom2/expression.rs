use std::fmt::{self, Debug};

use proc_macro::{Group, Span, TokenStream, TokenTree};

use crate::dom2::Node;
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
        Expression {
            stream: group.stream(),
            span: group.span(),
        }
    }
}

impl From<&str> for Expression {
    fn from(code: &str) -> Self{
       Expression {
            stream: code.tokenize(),
            span: Span::call_site(),
        }
    }
}
