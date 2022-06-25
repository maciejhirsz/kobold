use proc_macro::{Delimiter, Ident, TokenTree};
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;
use std::fmt::{self, Debug, Display};

use crate::parse::*;
use crate::parser::{into_quote, ParseError};
use crate::syntax::CssLabel;

pub struct Field {
    pub kind: FieldKind,
    pub typ: QuoteTokens,
    pub name: QuoteTokens,
    pub expr: QuoteTokens,
}

impl Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Foo")
            .field("kind", &self.kind)
            .field("typ", &DisplayDebug(&self.typ))
            .field("name", &DisplayDebug(&self.name))
            .field("expr", &DisplayDebug(&self.expr))
            .finish()
    }
}

struct DisplayDebug<T>(T);

impl<T: Display> Debug for DisplayDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub enum FieldKind {
    Html,
    AttrNode,
    AttrHoisted(QuoteTokens),
    Callback(String),
}

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Text(String),
    Expression, // variable arg, node ref arg
    Fragment(Vec<Node>),
}

impl Node {
    pub fn is_expression(&self) -> bool {
        matches!(self, Node::Expression)
    }

    pub fn is_fragment(&self) -> bool {
        matches!(self, Node::Fragment(_))
    }
}

#[derive(Debug)]
pub struct Element {
    pub tag: String,
    pub generics: Option<QuoteTokens>,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub children_raw: Option<QuoteTokens>,
    /// Element has been invoked with `..` spread at the end
    pub defaults: bool,
    /// Attribute fields that need to be hoisted into this element
    pub hoisted_attrs: Vec<QuoteTokens>,
}

impl Element {
    pub fn new(tag: String) -> Self {
        Element {
            tag,
            generics: None,
            attributes: Vec::new(),
            children: Vec::new(),
            children_raw: None,
            defaults: false,
            hoisted_attrs: Vec::new(),
        }
    }

    pub fn is_component(&self) -> bool {
        !self.tag.chars().next().unwrap().is_ascii_lowercase()
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub ident: Ident,
    pub value: AttributeValue,
}

#[derive(Debug)]
pub enum AttributeValue {
    Literal(QuoteTokens),
    Hoisted(QuoteTokens, QuoteTokens),
    Expression(QuoteTokens),
}

impl AttributeValue {
    pub fn from_group(name: &str, tokens: QuoteTokens) -> Self {
        // TODO: if the `tokens contains just a single literal,
        //       make it a literal value then as well.
        match name {
            "checked" => AttributeValue::Hoisted(
                quote! { bool },
                quote! { ::kobold::attribute::Checked(#tokens) },
            ),
            _ => AttributeValue::Expression(tokens),
        }
    }

    pub fn css_attribute_value(name: &str, stream: &mut ParseStream) -> Result<Self, ParseError> {
        let value = match stream.peek() {
            Some(TokenTree::Ident(_)) => {
                let css_label: CssLabel = stream.parse()?;

                AttributeValue::Literal(into_quote(css_label.into_literal()))
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let group = group.stream();
                stream.next();
                AttributeValue::from_group(&name, group.into())
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
