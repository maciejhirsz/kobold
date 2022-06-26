use arrayvec::ArrayString;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::dom2::Node;

pub fn generate(nodes: Vec<Node>) {
    let gen = Generator::default();
}

#[derive(Default)]
pub struct Generator {
    names: FieldGenerator,
    out: Transient,
}

pub struct JsSnippet {
    body: String,
    args: Vec<Ident>,
}

#[derive(Default)]
pub struct Transient {
    fields: Vec<Field>,
    // elements: Vec<DomNode>,
}

pub struct Field {
    name: Ident,
    typ: Ident,
    value: TokenStream,
    bounds: TokenStream,
    build: TokenStream,
    update: TokenStream,
}

// pub struct DomNode {
//     name: Ident,
//     js: JsSnippet,
//     hoisted: bool,
// }

pub enum DomNode {
    Rust(Ident),
    TextNode(String),
}

impl Generator {
    pub fn to_dom(&mut self, node: Node) -> DomNode {
        match node {
            Node::Expression(expr) => {
                let (typ, name) = self.names.next();

                let value = expr.stream.into();

                let bounds = quote!(#typ: ::kobold::Html,);
                let build = quote!(let #name = self.#name.build());
                let update = quote!(self.#name.update(&mut p.#name));

                self.out.fields.push(Field {
                    name: name.clone(),
                    typ,
                    value,
                    bounds,
                    build,
                    update,
                });

                DomNode::Rust(name)
            }
            Node::Text(lit) => {
                DomNode::TextNode(literal_to_string(lit))
            }
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

        // This gives us up to 456976 unique identifiers, should be enough :)
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

pub fn literal_to_string(lit: impl ToString) -> String {
    const QUOTE: &str = "\"";

    let stringified = lit.to_string();

    match stringified.chars().next() {
        // Take the string verbatim
        Some('"' | '\'') => stringified,
        _ => {
            let mut buf = String::with_capacity(stringified.len() + QUOTE.len() * 2);

            buf.extend([QUOTE, &stringified, QUOTE]);
            buf
        }
    }
}
