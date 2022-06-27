use std::fmt::{self, Display};

use arrayvec::ArrayString;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::dom2::{Expression, Node};

mod component;
mod element;

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
    fn add_expression(&mut self, value: TokenStream) -> FieldId {
        let (typ, name) = self.names.next();

        self.out.add(Field::Html {
            name,
            typ,
            value,
        })
    }
}

pub type FieldId = usize;

#[derive(Default)]
pub struct Transient {
    fields: Vec<Field>,
    // elements: Vec<DomNode>,
}

impl Transient {
    fn add(&mut self, field: Field) -> FieldId {
        let id = self.fields.len();

        self.fields.push(field);

        id
    }

    fn name(&self, id: FieldId) -> &Ident {
        match &self.fields[id] {
            Field::Html { name, .. } | Field::Attribute { name, .. } => name,
        }
    }
}

pub enum Field {
    Html {
        name: Ident,
        typ: Ident,
        value: TokenStream,
    },
    Attribute {
        name: Ident,
        typ: Ident,
        el: Ident,
        abi: TokenStream,
        value: TokenStream,
    },
}

impl Field {
    fn bounds(&self) -> TokenStream {
        match self {
            Field::Html { typ, .. } => quote! {
                #typ: ::kobold::Html,
            },
            Field::Attribute { typ, abi, .. } => quote! {
                #typ: ::kobold::Html,
                #typ::Product: ::kobold::attribute::AttributeProduct<Abi = #abi>,
            },
        }
    }

    fn build(&self) -> TokenStream {
        match self {
            Field::Html { name, .. } | Field::Attribute { name, .. } => quote! {
                let #name = self.#name.build();
            },
        }
    }

    fn update(&self) -> TokenStream {
        match self {
            Field::Html { name, .. } => quote! {
                self.#name.update(&mut p.#name);
            },
            Field::Attribute { name, el, .. } => quote! {
                self.#name.update(&mut p.#name, &p.#el);
            },
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
        let id = gen.add_expression(self.stream.into());

        DomNode::Variable(id)
    }
}

impl IntoGenerator for Node {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        match self {
            Node::Component(component) => component.into_gen(gen),
            Node::Expression(expr) => expr.into_gen(gen),
            Node::Text(lit) => DomNode::JsNode(JsNode::TextNode{ text: JsString(lit) }),
            _ => unimplemented!(),
        }
    }
}

#[derive(Default)]
pub struct FieldGenerator {
    count: usize,
}

impl FieldGenerator {
    fn next(&mut self) -> (Ident, Ident) {
        const LETTERS: usize = 26;

        // This gives us up to 26**4 = 456976 unique identifiers, should be enough :)
        let mut buf = ArrayString::<4>::new();
        let mut n = self.count;

        self.count += 1;

        loop {
            buf.push((u8::try_from(n % LETTERS).unwrap() + b'A') as char);

            n /= LETTERS;

            if n == 0 {
                break;
            }
        }

        let typ = Ident::new(&buf, Span::call_site());

        buf.make_ascii_lowercase();

        let name = Ident::new(&buf, Span::call_site());

        (typ, name)
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
