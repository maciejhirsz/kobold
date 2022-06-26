use std::vec::IntoIter as VecIter;

use arrayvec::ArrayString;
use proc_macro::{Ident, Span, TokenStream};

use crate::dom2::Node;

pub fn generate(nodes: Vec<Node>) {
    let mut nodes = nodes.into_iter();

    let gen = Generator::new(&mut nodes);
}

pub struct Generator<'a> {
    nodes: &'a mut VecIter<Node>,
    names: FieldGenerator,
}

pub struct JsSnippet {
    body: String,
    args: Vec<Ident>,
}

pub struct Transient {
    fields: Vec<Field>,
    elements: Vec<Element>,
}

pub struct Field {
    name: Ident,
    typ: Ident,
    bounds: TokenStream,
    build: TokenStream,
    update: TokenStream,
}

pub struct Element {
    name: Ident,
    js: JsSnippet,
    hoisted: bool,
}

impl<'a> Generator<'a> {
    pub fn new(nodes: &'a mut VecIter<Node>) -> Self {
        Generator {
            nodes,
            names: FieldGenerator::default(),
        }
    }

    pub fn generate(&mut self) -> Transient {
        unimplemented!()
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
