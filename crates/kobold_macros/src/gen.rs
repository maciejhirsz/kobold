// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{Debug, Write};
use std::hash::Hash;

use arrayvec::ArrayString;
use tokens::{Ident, TokenStream};

use crate::dom::{Expression, Node};
use crate::itertools::IteratorExt;
use crate::tokenize::prelude::*;

mod component;
mod element;
mod fragment;
mod transient;

pub use element::JsElement;
pub use fragment::{append, JsFragment};
pub use transient::{Anchor, Field, FieldKind, Hint, Transient};
pub use transient::{JsArgument, JsFnName, JsFunction, JsModule, JsString};

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
    fn add_field(&mut self, value: TokenStream) -> &mut Field {
        let name = self.names.next();

        self.out.fields.push(Field::new(name, value));
        self.out.fields.last_mut().unwrap()
    }

    fn add_hint(&mut self, name: Ident, typ: impl Tokenize) {
        self.out.hints.push(Hint {
            name,
            typ: typ.tokenize(),
        });
    }

    fn add_attr_hint(&mut self, name: Ident, lt: &str, attr: &str) {
        self.add_hint(
            name,
            format_args!("impl ::kobold::attribute::Attribute<{lt} ::kobold::attribute::{attr}>"),
        );
    }

    fn hoist(&mut self, node: DomNode) -> Option<JsFnName> {
        use std::hash::Hasher;

        let (var, body, args, anchor) = match node {
            DomNode::Variable(_) => return None,
            DomNode::TextNode(text) => {
                let body = format!("return document.createTextNode({text});\n");
                let var = self.names.next_el();

                (var, body, Vec::new(), Anchor::Node)
            }
            DomNode::Element(JsElement {
                tag,
                typ,
                var,
                code,
                args,
                hoisted: _,
            }) => {
                let create_tag = tag.to_js_create_element();

                let body = if code.is_empty() {
                    format!("return {create_tag};\n")
                } else {
                    format!("let {var}={create_tag};\n{code}return {var};\n")
                };

                (var, body, args, Anchor::Element(typ))
            }
            DomNode::Fragment(JsFragment { var, code, args }) => {
                assert!(
                    !code.is_empty(),
                    "Document fragment mustn't be empty, this is a bug"
                );

                (var, code, args, Anchor::Fragment)
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

        self.out
            .js
            .functions
            .push(JsFunction { name, anchor, args });

        Some(name)
    }
}

trait IntoGenerator {
    fn into_gen(self, gen: &mut Generator) -> DomNode;
}

impl IntoGenerator for Expression {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let field = gen.add_field(self.stream);

        if self.is_static {
            field.kind = FieldKind::StaticView;
        }

        DomNode::Variable(field.name)
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
