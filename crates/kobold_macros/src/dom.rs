use proc_macro::Ident;
use proc_macro2::TokenStream as QuoteTokens;
use std::fmt::{self, Debug, Display};

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
    Attr,
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
    pub defaults: bool,
}

impl Element {
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
    Text(String),
    Expression(QuoteTokens),
}
