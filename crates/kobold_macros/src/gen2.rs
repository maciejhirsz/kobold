use std::fmt::{self, Display};

use arrayvec::ArrayString;
use proc_macro::{Ident, TokenStream};

use crate::dom2::{Expression, Node};
use crate::parse::TokenStreamExt;

mod component;
mod element;

pub type Short = ArrayString<8>;

pub use element::JsNode;

pub fn generate(nodes: Vec<Node>) {
    let gen = Generator::default();
}

#[derive(Default)]
pub struct Generator {
    names: FieldGenerator,
    out: Transient,
}

impl Generator {
    fn add_expression(&mut self, value: impl Into<TokenStream>) -> (FieldId, &Short) {
        let name = self.names.next();

        self.out.add(Field::Html {
            name,
            value: value.into(),
        })
    }
}

pub type FieldId = usize;

#[derive(Default)]
pub struct Transient {
    fields: Vec<Field>,
    elements: usize,
    // elements: Vec<DomNode>,
}

impl Transient {
    fn add(&mut self, field: Field) -> (FieldId, &Short) {
        let id = self.fields.len();

        self.fields.push(field);

        (id, self.name(id))
    }

    fn name(&self, id: FieldId) -> &Short {
        match &self.fields[id] {
            Field::Html { name, .. } | Field::Attribute { name, .. } => name,
        }
    }

    fn next_el(&mut self) -> usize {
        let el = self.elements;

        self.elements += 1;

        el
    }
}

pub enum Field {
    Html {
        name: Short,
        value: TokenStream,
    },
    Attribute {
        name: Short,
        el: Ident,
        abi: TokenStream,
        value: TokenStream,
    },
}

impl Field {
    fn to_bounds(&self, stream: &mut TokenStream) {
        match self {
            Field::Html { name, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                write!(stream, "{typ}: ::kobold:Html,");
            }
            Field::Attribute { name, abi, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                write!(
                    stream,
                    "{typ}: ::kobold:Html,\
                    {typ}::Product: ::kobold::attribute::AttributeProduct<Abi = {abi}>,"
                );
            }
        }
    }

    fn build(&self, stream: &mut TokenStream) {
        let name = match self {
            Field::Html { name, .. } | Field::Attribute { name, .. } => name,
        };

        write!(stream, "let {name} = self.{name}.build();");
    }

    fn update(&self, stream: &mut TokenStream) {
        match self {
            Field::Html { name, .. } => {
                write!(stream, "self.{name}.update(&mut p.{name});");
            }
            Field::Attribute { name, el, .. } => {
                write!(stream, "self.{name}.update(&mut p.{name}, &p.{el});");
            }
        }
    }
}

pub enum DomNode {
    /// This node represents a variable, index mapping to a `Field` on `Transient`
    Variable(FieldId),
    /// This node is an element that can be constructed in JavaScript
    JsNode(JsNode),
}

trait IntoGenerator {
    fn into_gen(self, gen: &mut Generator) -> DomNode;
}

impl IntoGenerator for Expression {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let (id, _) = gen.add_expression(self.stream);

        DomNode::Variable(id)
    }
}

impl IntoGenerator for Node {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        match self {
            Node::Component(component) => component.into_gen(gen),
            Node::Expression(expr) => expr.into_gen(gen),
            Node::Text(lit) => DomNode::JsNode(JsNode::TextNode {
                text: JsString(lit),
            }),
            _ => unimplemented!(),
        }
    }
}

#[derive(Default)]
pub struct FieldGenerator {
    count: usize,
}

impl FieldGenerator {
    fn next(&mut self) -> Short {
        const LETTERS: usize = 26;

        let mut buf = Short::new();
        let mut n = self.count;

        self.count += 1;

        loop {
            buf.push((u8::try_from(n % LETTERS).unwrap() + b'a') as char);

            n /= LETTERS;

            if n == 0 {
                break;
            }
        }

        buf
    }
}

pub struct JsString(proc_macro::Literal);

impl Display for JsString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringified = self.0.to_string();

        match stringified.chars().next() {
            // Take the string verbatim
            Some('"' | '\'') => f.write_str(&stringified),
            // Add quotes
            _ => write!(f, "\"{stringified}\""),
        }
    }
}
