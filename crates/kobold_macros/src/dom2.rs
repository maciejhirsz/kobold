use std::fmt::{self, Debug};
use std::str::FromStr;

use proc_macro::{Delimiter, Ident, Literal, Spacing, Span, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::syntax::CssLabel;

mod shallow;

pub use shallow::{ShallowNode, ShallowNodeIter, ShallowStream, TagName, TagNesting};

pub fn parse(tokens: TokenStream) -> Result<Vec<Node>, ParseError> {
    let mut stream = tokens.parse_stream().into_shallow_stream();

    let mut nodes = Vec::new();

    while let Some(node) = Node::parse(&mut stream)? {
        nodes.push(node);
    }

    if nodes.is_empty() {
        return Err(ParseError::new("Empty html! invocation", Span::call_site()));
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

pub struct Expression(pub TokenStream);

impl Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Expression> for Node {
    fn from(expr: Expression) -> Node {
        Node::Expression(expr)
    }
}

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub span: Span,
    pub path: TokenStream,
    pub generics: Option<TokenStream>,
    pub props: Vec<Property>,
    pub spread: Option<TokenStream>,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug)]
pub struct HtmlElement {
    pub name: String,
    pub span: Span,
    pub attributes: Vec<Attribute>,
    pub children: Option<Vec<Node>>,
}

#[derive(Debug)]
pub struct Property {
    pub name: Ident,
    pub expr: Expression,
}

#[derive(Debug)]
pub struct Attribute {
    pub name: Ident,
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
    fn parse(stream: &mut ShallowStream) -> Result<Option<Self>, ParseError> {
        let tag = match stream.next() {
            Some(Ok(ShallowNode::Tag(tag))) => tag,
            Some(Ok(ShallowNode::Literal(lit))) => {
                return Ok(Some(Node::Text(lit)));
            }
            Some(Ok(ShallowNode::Expression(expr))) => {
                return Ok(Some(Expression(expr).into()));
            }
            Some(Err(error)) => {
                return Err(error.msg("Expected a tag, a string literal, or an {expression}"))
            }
            None => return Ok(None),
        };

        let children = match tag.nesting {
            TagNesting::SelfClosing => None,
            TagNesting::Opening => Some(Node::parse_children(&tag.name, stream)?),
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
                let mut spread = None;
                let mut props = Vec::new();

                while !content.end() {
                    if content.allow(('.', Spacing::Joint)) {
                        content.expect('.')?;

                        if let Some(TokenTree::Group(expr)) =
                            content.allow_consume(Delimiter::Brace)
                        {
                            spread = Some(expr.stream());
                        } else {
                            let expr = TokenStream::from_str("Default::default()").unwrap();

                            spread = Some(expr);
                        }

                        if let Some(tt) = content.next() {
                            return Err(ParseError::new("Not allowed after the .. spread", tt));
                        }

                        break;
                    }

                    props.push(content.parse()?);
                }

                Ok(Some(Node::Component(Component {
                    name,
                    span,
                    path,
                    generics,
                    props,
                    spread,
                    children,
                })))
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

                Ok(Some(Node::HtmlElement(HtmlElement {
                    name,
                    span,
                    attributes,
                    children,
                })))
            }
        }
    }

    fn parse_children(name: &TagName, stream: &mut ShallowStream) -> Result<Vec<Node>, ParseError> {
        let mut children = Vec::new();

        loop {
            if let Some(Ok(ShallowNode::Tag(tag))) = stream.peek() {
                if tag.is_closing(name) {
                    stream.next();
                    break;
                }
            }

            match Node::parse(stream)? {
                Some(node) => children.push(node),
                None => {
                    return Err(ParseError::new(
                        format!("Missing closing tag for {name}"),
                        name.span(),
                    ))
                }
            }
        }

        Ok(children)
    }
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
                expr: Expression(expr.stream()),
            });
        }

        let name = stream.parse()?;

        stream.expect('=')?;

        match stream.next() {
            Some(TokenTree::Group(expr)) if expr.delimiter() == Delimiter::Brace => Ok(Property {
                name,
                expr: Expression(expr.stream()),
            }),
            Some(TokenTree::Literal(lit)) => {
                let mut expr = TokenStream::new();

                expr.push(lit);

                Ok(Property {
                    name,
                    expr: Expression(expr),
                })
            }
            Some(TokenTree::Ident(b)) if b.str_eq("true") || b.str_eq("false") => {
                let mut expr = TokenStream::new();

                expr.push(b);

                Ok(Property {
                    name,
                    expr: Expression(expr),
                })
            }
            _ => Err(ParseError::new(
                "Component properties must contain {expressions} or literals",
                name.span(),
            )),
        }
    }
}

impl AttributeValue {
    pub fn parse_css_value(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let value = match stream.peek() {
            Some(TokenTree::Ident(_)) => {
                let css_label: CssLabel = stream.parse()?;

                AttributeValue::Literal(css_label.into_literal())
            }
            Some(TokenTree::Group(expr)) if expr.delimiter() == Delimiter::Brace => {
                AttributeValue::Expression(Expression(expr.stream()))
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

            let name = inner.parse()?;

            if let Some(tt) = inner.next() {
                return Err(ParseError::new(
                    "Shorthand expressions can only contain a single identifier",
                    tt,
                ));
            }

            return Ok(Attribute {
                name,
                value: Expression(expr.stream()).into(),
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
            Some(TokenTree::Group(expr)) if expr.delimiter() == Delimiter::Brace => Ok(Attribute {
                name,
                value: Expression(expr.stream()).into(),
            }),
            Some(TokenTree::Literal(lit)) => Ok(Attribute {
                name,
                value: AttributeValue::Literal(lit),
            }),
            Some(TokenTree::Ident(b)) if b.str_eq("true") || b.str_eq("false") => Ok(Attribute {
                name,
                value: AttributeValue::Boolean(b),
            }),
            _ => Err(ParseError::new(
                "Element attributes must contain {expressions} or literals",
                name.span(),
            )),
        }
    }
}
