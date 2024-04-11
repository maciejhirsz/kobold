// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tokens::{Delimiter, Ident, Literal, Span, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::syntax::CssLabel;
use crate::tokenize::prelude::*;

mod els;
mod expression;
mod shallow;

pub use els::ElementTag;
pub use expression::Expression;
pub use shallow::{IsClosing, ShallowNode, ShallowNodeIter, ShallowStream, TagName, TagNesting};

pub fn parse(tokens: TokenStream) -> Result<Vec<Node>, ParseError> {
    let mut stream = tokens.parse_stream().into_shallow_stream();

    let mut nodes = Vec::new();

    while Node::parse(&mut stream, &mut nodes)? > 0 {}

    if nodes.is_empty() {
        return Err(ParseError::new("Empty view! invocation", Span::call_site()));
    }

    Ok(nodes)
}

#[derive(Debug)]
pub enum Node {
    HtmlElement(HtmlElement),
    Component(Component),
    Text(Literal),
    Expression(Expression),
}

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub span: Span,
    pub path: TokenStream,
    pub generics: Option<TokenStream>,
    pub props: Vec<Property>,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug)]
pub struct HtmlElement {
    pub name: ElementTag,
    pub span: Span,
    pub classes: Vec<CssValue>,
    pub attributes: Vec<Attribute>,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug)]
pub struct Property {
    pub name: Ident,
    pub expr: Expression,
}

#[derive(Debug)]
pub enum CssValue {
    Literal(Literal),
    Expression(Expression),
}

#[derive(Debug)]
pub struct Attribute {
    pub name: CssLabel,
    pub value: AttributeValue,
}

#[derive(Debug)]
pub enum AttributeValue {
    Literal(Literal),
    Boolean(Ident),
    Expression(Expression),
}

impl From<Expression> for AttributeValue {
    fn from(expr: Expression) -> AttributeValue {
        AttributeValue::Expression(expr)
    }
}

impl Node {
    fn parse(stream: &mut ShallowStream, parent: &mut Vec<Node>) -> Result<usize, ParseError> {
        let tag = match stream.next() {
            Some(Ok(ShallowNode::Tag(tag))) => tag,
            Some(Ok(ShallowNode::Literal(lit))) => {
                parent.push(Node::Text(lit));
                return Ok(1);
            }
            Some(Ok(ShallowNode::Expression(expr))) => {
                parent.push(Expression::try_from(expr)?.into());
                return Ok(1);
            }
            Some(Err(error)) => return Err(error),
            None => return Ok(0),
        };

        let children = match tag.nesting {
            TagNesting::SelfClosing => Children::None,
            TagNesting::Opening => Node::parse_children(&tag.name, stream)?,
            TagNesting::Closing => {
                return Err(ParseError::new(
                    format!("Unexpected closing tag {}", tag.name),
                    tag.name,
                ));
            }
        };

        match tag.name {
            TagName::Component {
                name,
                span,
                path,
                generics,
            } => {
                let mut content = tag.content.parse_stream();
                let mut props = Vec::new();

                while !content.end() {
                    props.push(content.parse()?);
                }

                let (children, tail) = match children {
                    Children::None => (None, Vec::new()),
                    Children::Explicit(children) => (Some(children), Vec::new()),
                    Children::Implicit(tail) => (None, tail),
                };

                let count = 1 + tail.len();

                parent.push(Node::Component(Component {
                    name,
                    span,
                    path,
                    generics,
                    props,
                    children,
                }));

                parent.extend(tail);

                Ok(count)
            }
            TagName::HtmlElement { name, span } => {
                let mut content = tag.content.parse_stream();
                let mut classes = Vec::new();
                let mut attributes = Vec::new();

                loop {
                    if content.allow_consume('.').is_some() {
                        classes.push(content.parse()?);
                    } else if let Some(hash) = content.allow_consume('#') {
                        let name = CssLabel {
                            label: "id".into(),
                            ident: Ident::new("id", hash.span()),
                        };
                        let value: CssValue = content.parse()?;

                        attributes.push(Attribute {
                            name,
                            value: value.into(),
                        })
                    } else {
                        break;
                    }
                }

                while !content.end() {
                    let attr: Attribute = content.parse()?;

                    if attr.name.label == "class" {
                        classes.push(CssValue::try_from(attr.value)?);
                    } else {
                        attributes.push(attr);
                    }
                }

                let children = match children {
                    Children::None => None,
                    Children::Explicit(children) | Children::Implicit(children) => Some(children),
                };

                parent.push(Node::HtmlElement(HtmlElement {
                    name,
                    span,
                    classes,
                    attributes,
                    children,
                }));

                Ok(1)
            }
        }
    }

    fn parse_children(name: &TagName, stream: &mut ShallowStream) -> Result<Children, ParseError> {
        let mut children = Vec::new();
        let mut explicit = false;

        loop {
            if let Some(Ok(ShallowNode::Tag(tag))) = stream.peek() {
                match tag.is_closing(name) {
                    IsClosing::Explicit => {
                        stream.next();
                        explicit = true;
                        break;
                    }
                    IsClosing::Implicit => {
                        break;
                    }
                    IsClosing::No => (),
                }
            }

            if Node::parse(stream, &mut children)? == 0 {
                break;
            }
        }

        if children.is_empty() {
            Ok(Children::None)
        } else if explicit {
            Ok(Children::Explicit(children))
        } else {
            Ok(Children::Implicit(children))
        }
    }
}

enum Children {
    None,
    Explicit(Vec<Node>),
    Implicit(Vec<Node>),
}

impl Parse for Property {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        // Allow expression shorthand
        if let Some(TokenTree::Group(expr)) = stream.allow_consume(Delimiter::Brace) {
            let mut inner = expr.stream().parse_stream();

            let name = inner.parse()?;

            if let Some(tt) = inner.next() {
                return Err(ParseError::new(
                    "Shorthand expressions can only contain a single identifier",
                    tt,
                ));
            }

            return Ok(Property {
                name,
                expr: Expression::try_from(expr)?,
            });
        }

        let name = stream.parse()?;

        stream.expect('=')?;

        match stream.next() {
            Some(tt) if tt.is('{') || tt.is(Lit) => Ok(Property {
                name,
                expr: Expression::try_from(tt)?,
            }),
            Some(TokenTree::Ident(b)) if b.one_of(["true", "false"]) => Ok(Property {
                name,
                expr: Expression::try_from(TokenTree::from(b))?,
            }),
            _ => Err(ParseError::new(
                "Component properties must contain {expressions} or literals",
                name.span(),
            )),
        }
    }
}

impl CssValue {
    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            CssValue::Literal(lit) => Some(lit),
            CssValue::Expression(_) => None,
        }
    }

    pub fn is_literal(&self) -> bool {
        !self.is_expression()
    }

    pub fn is_expression(&self) -> bool {
        match self {
            CssValue::Literal(_) => false,
            CssValue::Expression(_) => true,
        }
    }
}

impl Parse for CssValue {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        if let Some(expr) = stream.allow_consume('{') {
            return Ok(CssValue::Expression(Expression::try_from(expr)?));
        }

        let css_label: CssLabel = stream
            .parse()
            .map_err(|err| err.msg("Expected identifier or an {expression}"))?;

        Ok(CssValue::Literal(css_label.into_literal()))
    }
}

impl TryFrom<AttributeValue> for CssValue {
    type Error = ParseError;

    fn try_from(value: AttributeValue) -> Result<CssValue, ParseError> {
        match value {
            AttributeValue::Literal(lit) => Ok(CssValue::Literal(lit)),
            AttributeValue::Expression(expr) => Ok(CssValue::Expression(expr)),
            AttributeValue::Boolean(b) => Err(ParseError::new(
                "Cannot assign bool to this attribute",
                b.span(),
            )),
        }
    }
}

impl From<CssValue> for AttributeValue {
    fn from(value: CssValue) -> AttributeValue {
        match value {
            CssValue::Literal(lit) => AttributeValue::Literal(lit),
            CssValue::Expression(expr) => AttributeValue::Expression(expr),
        }
    }
}

impl Parse for Attribute {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        if let Some(TokenTree::Group(expr)) = stream.allow_consume('{') {
            let mut inner = expr.stream().parse_stream();

            let name = inner.parse()?;

            if let Some(tt) = inner.next() {
                return Err(ParseError::new(
                    "Shorthand expressions can only contain a single identifier",
                    tt,
                ));
            }

            return Ok(Attribute {
                name,
                value: Expression::try_from(expr)?.into(),
            });
        }

        if stream.allow('.') || stream.allow('#') {
            return Err(ParseError::new(
                "CSS-like class and id attributes must be defined before other attributes",
                stream.next(),
            ));
        }

        let name = stream.parse()?;

        stream.expect('=')?;

        match stream.next() {
            Some(TokenTree::Literal(lit)) => Ok(Attribute {
                name,
                value: AttributeValue::Literal(lit),
            }),
            Some(TokenTree::Ident(b)) if b.one_of(["true", "false"]) => Ok(Attribute {
                name,
                value: AttributeValue::Boolean(b),
            }),
            Some(tt) if tt.is('{') => Ok(Attribute {
                name,
                value: Expression::try_from(tt)?.into(),
            }),
            _ => Err(ParseError::new(
                "Element attributes must contain {expressions} or literals",
                name.ident.span(),
            )),
        }
    }
}
