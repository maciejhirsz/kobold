use proc_macro::Ident;
use proc_macro2::TokenStream as QuoteTokens;

#[derive(Debug)]
pub struct Field {
    pub typ: QuoteTokens,
    pub name: QuoteTokens,
    pub expr: QuoteTokens,
}

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Text(String),
    Expression, // variable arg, node ref arg
    Fragment(Vec<Node>),
}

#[derive(Debug)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
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
