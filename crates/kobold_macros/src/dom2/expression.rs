use std::fmt::{self, Debug};
use std::str::FromStr;

use proc_macro::{Group, Span, TokenStream, TokenTree};

use crate::dom2::Node;

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
        let stream = TokenStream::from(tt);

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

impl Expression {
    pub fn from_str(code: &str) -> Expression {
        let stream = TokenStream::from_str(code).unwrap();

        Expression {
            stream,
            span: Span::call_site(),
        }
    }
}
