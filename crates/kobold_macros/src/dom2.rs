use proc_macro::{Delimiter, Ident, Literal, Spacing, Span, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::syntax::{CssLabel, Tag, TagName, TagNesting};

pub enum Node {
    HtmlElement(HtmlElement),
    Component(Component),
    Text(String),
    Expression(TokenStream),
    Fragment(Vec<Node>),
}

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

pub struct Component {
    pub name: String,
    pub path: TokenStream,
    pub generics: Option<TokenStream>,
    pub props: Vec<Property>,
    pub defaults: bool,
}

pub struct Property {
    pub name: Ident,
    pub expr: TokenStream,
}

pub struct HtmlElement {
    pub name: String,
    pub span: Span,
    pub attributes: Vec<Attribute>,
}

pub struct Attribute {
    pub name: Ident,
    pub value: AttributeValue,
}

pub enum AttributeValue {
    Literal(TokenTree),
    Expression(TokenStream),
}

pub fn shallow_parse(mut tokens: TokenStream) -> Result<Vec<ShallowNode>, ParseError> {
    tokens.parse_stream().parse()
}

impl Node {
    fn parse(shallow: &mut impl Iterator<Item = ShallowNode>) -> Result<Self, ParseError> {
        let mut tag = match shallow.next() {
            Some(ShallowNode::Tag(tag)) => tag,
            _ => unimplemented!(),
        };

        if tag.nesting == TagNesting::Closing {
            return Err(ParseError::new(
                "Unexpected closing tag",
                Some(tag.name.into_spanned_token()),
            ));
        }

        match tag.name {
            TagName::Component {
                name,
                path,
                generics,
            } => {
                let mut content = tag.content.parse_stream();
                let mut defaults = false;
                let mut props = Vec::new();

                while !content.end() {
                    if content.allow(('.', Spacing::Joint)) {
                        content.expect('.')?;

                        if let Some(tt) = content.next() {
                            return Err(ParseError::new(
                                "Not allowed after the .. default spread",
                                Some(tt),
                            ));
                        }

                        defaults = true;
                        break;
                    }

                    props.push(content.parse()?);
                }

                if tag.nesting == TagNesting::Opening {
                    // TODO: Children
                }

                Ok(Node::Component(Component {
                    name,
                    path,
                    generics,
                    props,
                    defaults,
                }))
            }
            TagName::HtmlElement { name, span } => {
                let mut content = tag.content.parse_stream();
                let mut attributes = Vec::new();

                loop {
                    let name = if let Some(class) = content.allow_consume('.') {
                        Ident::new("class", class.span())
                    } else if let Some(id) = content.allow_consume('#') {
                        Ident::new("id", id.span())
                    } else {
                        break;
                    };

                    attributes.push(Attribute {
                        name,
                        value: AttributeValue::parse_css_value(&mut content)?,
                    });
                }

                while !content.end() {
                    attributes.push(content.parse()?);
                }

                if tag.nesting == TagNesting::Opening {
                    // TODO: Children
                }

                Ok(Node::HtmlElement(HtmlElement {
                    name,
                    span,
                    attributes,
                }))
            }
        }
    }
}

impl Parse for Property {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        // Allow expression shorthand
        if let Some(TokenTree::Group(expr)) = stream.allow_consume(Delimiter::Brace) {
            let mut inner = expr.stream().parse_stream();

            let name: Ident = inner.parse()?;

            if let Some(tt) = inner.next() {
                return Err(ParseError::new(
                    "Shorthand expressions can only contain a single identifier",
                    Some(tt),
                ));
            }

            return Ok(Property {
                name,
                expr: expr.stream(),
            });
        }

        let name: Ident = stream.parse()?;

        stream.expect('=')?;

        match stream.next() {
            Some(TokenTree::Group(expr)) if expr.delimiter() == Delimiter::Brace => Ok(Property {
                name,
                expr: expr.stream(),
            }),
            Some(TokenTree::Literal(lit)) => {
                let mut expr = TokenStream::new();

                expr.push(lit);

                Ok(Property { name, expr })
            }
            _ => Err(ParseError::new(
                "Component properties must contain {expressions} or literals",
                Some(name.into()),
            )),
        }
    }
}

impl AttributeValue {
    pub fn parse_css_value(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let value = match stream.peek() {
            Some(TokenTree::Ident(_)) => {
                let css_label: CssLabel = stream.parse()?;

                AttributeValue::Literal(css_label.into_literal().into())
            }
            Some(TokenTree::Group(expr)) if expr.delimiter() == Delimiter::Brace => {
                AttributeValue::Expression(expr.stream())
            }
            _ => {
                return Err(ParseError::new(
                    "Expected identifier or an {expression}",
                    stream.next(),
                ))
            }
        };

        Ok(value)
    }
}

impl Parse for Attribute {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        if let Some(TokenTree::Group(expr)) = stream.allow_consume(Delimiter::Brace) {
            let mut inner = expr.stream().parse_stream();

            let name: Ident = inner.parse()?;

            if let Some(tt) = inner.next() {
                return Err(ParseError::new(
                    "Shorthand expressions can only contain a single identifier",
                    Some(tt),
                ));
            }

            return Ok(Attribute {
                name,
                value: AttributeValue::Expression(expr.stream()),
            });
        }

        if stream.allow('.') || stream.allow('#') {
            return Err(ParseError::new(
                "CSS-like class and id attributes must be defined before other attributes",
                stream.next(),
            ));
        }

        let name: Ident = stream.parse()?;

        stream.expect('=');

        match stream.next() {
            Some(TokenTree::Group(expr)) if expr.delimiter() == Delimiter::Brace => Ok(Attribute {
                name,
                value: AttributeValue::Expression(expr.stream()),
            }),
            Some(TokenTree::Literal(lit)) => Ok(Attribute {
                name,
                value: AttributeValue::Literal(lit.into()),
            }),
            _ => Err(ParseError::new(
                "Element attributes must contain {expressions} or literals",
                Some(name.into()),
            )),
        }
    }
}
