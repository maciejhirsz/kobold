use std::fmt::{Debug, Write};
use std::hash::Hash;

use arrayvec::ArrayString;
use proc_macro::TokenStream;

use crate::dom::{Expression, Node};
use crate::itertools::IteratorExt;
use crate::tokenize::prelude::*;

mod component;
mod element;
mod fragment;
mod transient;

pub use element::JsElement;
pub use fragment::{append, JsFragment};
pub use transient::{Abi, Field, JsArgument, JsFnName, JsFunction, JsModule, JsString, Transient};

// Short string for auto-generated variable names
pub type Short = ArrayString<4>;

pub enum DomNode {
    Variable(Short),
    TextNode(JsString),
    Element(JsElement),
    Fragment(JsFragment),
}

pub fn generate(mut nodes: Vec<Node>) -> Transient {
    let mut gen = Generator::default();

    let dom_node = if nodes.len() == 1 {
        nodes.remove(0).into_gen(&mut gen)
    } else {
        nodes.into_gen(&mut gen)
    };

    gen.hoist(dom_node);
    gen.out
}

#[derive(Default)]
pub struct Generator {
    names: NameGenerator,
    out: Transient,
}

impl Generator {
    fn set_js_type(&mut self, ty: &'static str) {
        if self.out.js_type.is_some() {
            return;
        }

        self.out.js_type = Some(ty);
    }

    fn add_expression(&mut self, value: TokenStream) -> Short {
        let name = self.names.next();

        self.out.fields.push(Field::View { name, value });

        name
    }

    fn add_attribute(&mut self, el: Short, abi: Abi, value: TokenStream) -> Short {
        let name = self.names.next();

        self.out.fields.push(Field::Attribute {
            name,
            el,
            abi,
            value,
        });

        name
    }

    fn hoist(&mut self, node: DomNode) -> Option<JsFnName> {
        use std::hash::Hasher;

        let (var, body, args, constructor) = match node {
            DomNode::Variable(_) => return None,
            DomNode::TextNode(text) => {
                let body = format!("return document.createTextNode({text});\n");
                let var = self.names.next_el();

                (var, body, Vec::new(), "Element::new")
            }
            DomNode::Element(JsElement {
                tag,
                var,
                code,
                args,
                hoisted: _,
            }) => {
                let body = if code.is_empty() {
                    format!("return document.createElement(\"{tag}\");\n")
                } else {
                    format!("let {var}=document.createElement(\"{tag}\");\n{code}return {var};\n")
                };

                (var, body, args, "Element::new")
            }
            DomNode::Fragment(JsFragment { var, code, args }) => {
                assert!(
                    !code.is_empty(),
                    "Document fragment mustn't be empty, this is a bug"
                );

                (var, code, args, "Element::new_fragment_raw")
            }
        };

        self.out.els.push(var);

        let mut hasher = fnv::FnvHasher::default();
        var.hash(&mut hasher);
        body.hash(&mut hasher);

        let hash = hasher.finish();
        let name = JsFnName::try_from(format_args!("__{var}_{hash:016x}")).unwrap();

        let js_args = args.iter().map(|a| a.name).join(",");

        let _ = write!(
            self.out.js.code,
            "export function {name}({js_args}) {{\
                \n{body}\
            }}\n\
            "
        );

        self.out.js.functions.push(JsFunction {
            name,
            constructor,
            args,
        });

        Some(name)
    }
}

trait IntoGenerator {
    fn into_gen(self, gen: &mut Generator) -> DomNode;
}

impl IntoGenerator for Expression {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let name = gen.add_expression(self.stream);

        DomNode::Variable(name)
    }
}

impl IntoGenerator for Node {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        match self {
            Node::Component(component) => component.into_gen(gen),
            Node::HtmlElement(element) => element.into_gen(gen),
            Node::Expression(expr) => expr.into_gen(gen),
            Node::Text(lit) => DomNode::TextNode(JsString(lit)),
        }
    }
}

#[derive(Default, Debug)]
pub struct NameGenerator {
    variables: usize,
    elements: usize,
}

impl NameGenerator {
    /// Generate next variable name, `a` to `z` lower case
    fn next(&mut self) -> Short {
        const LETTERS: usize = 26;

        let mut buf = Short::new();
        let mut n = self.variables;

        self.variables += 1;

        loop {
            buf.push((u8::try_from(n % LETTERS).unwrap() + b'a') as char);

            n /= LETTERS;

            if n == 0 {
                break;
            }
        }

        buf
    }

    /// Generate next element name, integer prefixed with `e`
    fn next_el(&mut self) -> Short {
        let n = self.elements;

        self.elements += 1;

        let mut var = Short::new();

        let _ = write!(var, "e{n}");

        var
    }
}
