//! Converting [`ParseStream`](parse::ParseStream) into [`ShallowStream`](ShallowStream).
//!
//! This is an intermediate representation of the syntax to make the final parsing of
//! nested elements easier.

use std::fmt::{self, Display, Write};

use proc_macro::{Group, Ident, Literal, Spacing, Span, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::syntax::Generics;

pub type ShallowStream = std::iter::Peekable<ShallowNodeIter>;

#[derive(Debug)]
pub enum ShallowNode {
    Tag(Tag),
    Literal(Literal),
    Expression(Group),
}

impl Parse for ShallowNode {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        if let Some(TokenTree::Group(expr)) = stream.allow_consume('{') {
            let mut try_lit = expr.stream().parse_stream();

            if let Some(TokenTree::Literal(lit)) = try_lit.allow_consume(Lit) {
                if try_lit.end() {
                    return Ok(ShallowNode::Literal(lit));
                }
            }

            return Ok(ShallowNode::Expression(expr));
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

/// Tag name for an element, either HTML element such as `div`, or a component `Foo`.
#[derive(Debug)]
pub enum TagName {
    HtmlElement {
        name: String,
        span: Span,
    },
    Component {
        name: String,
        span: Span,
        path: TokenStream,
        generics: Option<TokenStream>,
    },
}

impl PartialEq for TagName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TagName::HtmlElement { name: l, .. }, TagName::HtmlElement { name: r, .. }) => l == r,
            (TagName::Component { name: l, .. }, TagName::Component { name: r, .. }) => l == r,
            _ => false,
        }
    }
}

impl TagName {
    pub fn span(&self) -> Span {
        match self {
            TagName::HtmlElement { span, .. } => *span,
            TagName::Component { span, .. } => *span,
        }
    }
}

impl IntoSpan for TagName {
    fn into_span(self) -> Span {
        self.span()
    }
}

impl Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TagName::HtmlElement { name, .. } => name,
            TagName::Component { name, .. } => name,
        };

        f.write_str(name)
    }
}

impl Parse for TagName {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut ident: Ident = stream.parse()?;
        let mut name = ident.to_string();
        let mut span = ident.span();

        if name.as_bytes()[0].is_ascii_lowercase() && !stream.allow((':', Spacing::Joint)) {
            return Ok(TagName::HtmlElement { name, span });
        }

        let mut path = TokenStream::new();

        path.push(ident);

        while let Some(colon) = stream.allow_consume((':', Spacing::Joint)) {
            path.push(colon);
            path.push(stream.expect(':')?);

            ident = stream.parse()?;
            span = ident.span();

            write!(&mut name, "::{ident}").unwrap();

            path.push(ident);
        }

        let mut generics = None;

        if stream.allow('<') {
            generics = Some(Generics::parse(stream)?.tokens);
        }

        Ok(TagName::Component {
            name,
            span,
            path,
            generics,
        })
    }
}

/// Describes nesting behavior of a tag
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TagNesting {
    /// Opening tag `<...>`
    Opening,
    /// Closing tag `</...>`
    Closing,
    /// Self-closing tag `<.../>`
    SelfClosing,
}

/// Non-descript tag
#[derive(Debug)]
pub struct Tag {
    pub name: TagName,
    pub nesting: TagNesting,
    pub content: TokenStream,
}

impl Tag {
    pub fn is_closing(&self, opening: &TagName) -> bool {
        self.nesting == TagNesting::Closing && &self.name == opening
    }
}

impl Parse for Tag {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        stream.expect('<')?;

        let mut nesting = match stream.allow_consume('/') {
            Some(_) => {
                let name = stream.parse()?;

                stream.expect('>')?;

                return Ok(Tag {
                    name,
                    nesting: TagNesting::Closing,
                    content: TokenStream::new(),
                });
            }
            None => TagNesting::Opening,
        };

        let name = stream.parse()?;

        let mut content = TokenStream::new();

        while let Some(tt) = stream.next() {
            if tt.is('/') {
                if nesting == TagNesting::Opening {
                    nesting = TagNesting::SelfClosing;

                    stream.expect('>')?;

                    return Ok(Tag {
                        name,
                        nesting,
                        content,
                    });
                } else {
                    return Err(ParseError::new("Unexpected closing slash", tt));
                }
            }

            if tt.is('>') {
                return Ok(Tag {
                    name,
                    nesting,
                    content,
                });
            }

            content.push(tt);
        }

        Err(ParseError::new(
            format!("Missing closing > for {name}"),
            name,
        ))
    }
}
