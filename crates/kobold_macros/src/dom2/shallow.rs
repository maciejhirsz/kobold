//! Converting [`ParseStream`](parse::ParseStream) into [`ShallowStream`](ShallowStream).
//!
//! This is an intermediate representation of the syntax to make the final parsing of
//! nested elements easier.

use proc_macro::{Delimiter, Literal, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::syntax::Tag;

pub type ShallowStream = std::iter::Peekable<ShallowNodeIter>;

#[derive(Debug)]
pub enum ShallowNode {
    Tag(Tag),
    Literal(Literal),
    Expression(TokenStream),
}

impl Parse for ShallowNode {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        if let Some(TokenTree::Group(expr)) = stream.allow_consume(Delimiter::Brace) {
            let mut try_lit = expr.stream().parse_stream();

            if let Some(TokenTree::Literal(lit)) = try_lit.allow_consume(Lit) {
                if try_lit.end() {
                    return Ok(ShallowNode::Literal(lit));
                }
            }

            return Ok(ShallowNode::Expression(expr.stream()));
        }

        if let Some(TokenTree::Literal(lit)) = stream.allow_consume(Lit) {
            return Ok(ShallowNode::Literal(lit));
        }

        stream.parse().map(ShallowNode::Tag)
    }
}

pub struct ShallowNodeIter {
    stream: ParseStream,
}

impl ShallowNodeIter {
    pub fn new(stream: ParseStream) -> Self {
        ShallowNodeIter { stream }
    }
}

impl Iterator for ShallowNodeIter {
    type Item = Result<ShallowNode, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stream.end() {
            return None;
        }

        Some(self.stream.parse())
    }
}
