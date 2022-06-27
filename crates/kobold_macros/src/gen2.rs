use std::fmt::{self, Display, Write};
use std::hash::Hash;

use arrayvec::ArrayString;
use proc_macro::TokenStream;

use crate::dom2::{Expression, Node};
use crate::gen2::element::{JsElement, JsFragment};
use crate::parse::TokenStreamExt;

mod component;
mod element;

pub type Short = ArrayString<8>;
pub type JsFnName = ArrayString<{ 3 + 8 + 16 }>; // underscores + up to 8 bytes for el + hash

pub use element::JsNode;

pub fn generate(nodes: Vec<Node>) {
    let gen = Generator::default();
}

#[derive(Default)]
pub struct Generator {
    names: FieldGenerator,
    js: JsModule,
    out: Transient,
}

impl Generator {
    fn add_expression(&mut self, value: impl Into<TokenStream>) -> &Short {
        let name = self.names.next();

        self.out.add(Field::Html {
            name,
            value: value.into(),
        })
    }

    fn hoist(&mut self, node: JsNode) -> JsFnName {
        use std::hash::Hasher;

        let (var, body, args) = match node {
            JsNode::TextNode(text) => {
                let body = format!("return document.createTextNode({text});\n");
                let var = self.out.next_el();

                (var, body, Vec::new())
            }
            JsNode::Element(el) => {
                let JsElement {
                    tag,
                    var,
                    code,
                    args,
                } = el;

                let body = if code.is_empty() {
                    format!("return document.createElement('{tag}');\n")
                } else {
                    format!("let {var}=document.createElement('{tag}');\n{code}return {var};\n")
                };

                (var, body, args)
            }
            JsNode::Fragment(frag) => {
                let JsFragment { var, code, args } = frag;

                assert!(
                    !code.is_empty(),
                    "Document fragment mustn't be empty, this is a bug"
                );

                let body =
                    format!("let {var}=document.createDocumentFragment();\n{code}return {var};\n");

                (var, body, args)
            }
        };

        let mut hasher = fnv::FnvHasher::default();
        var.hash(&mut hasher);
        body.hash(&mut hasher);

        let hash = hasher.finish();
        let name = JsFnName::try_from(format_args!("__{var}_{hash:016x}")).unwrap();

        let js = &mut self.js.code;

        js.push_str("export function ");
        js.push_str(&name);
        js.push('(');
        {
            let mut args = args.iter();

            if let Some(arg) = args.next() {
                js.push_str(arg);

                for arg in args {
                    js.push(',');
                    js.push_str(arg);
                }
            }
        }
        js.push_str("){\n");
        js.push_str(&body);
        js.push_str("}\n");

        self.js.funtions.push(JsFunction { name, args });

        name
    }
}

#[derive(Default)]
pub struct JsModule {
    pub funtions: Vec<JsFunction>,
    pub code: String,
}

pub struct JsFunction {
    pub name: JsFnName,
    pub args: Vec<Short>,
}

pub type FieldId = usize;

#[derive(Default)]
pub struct Transient {
    fields: Vec<Field>,
    elements: usize,
}

impl Transient {
    fn add(&mut self, field: Field) -> &Short {
        let id = self.fields.len();

        self.fields.push(field);
        self.name(id)
    }

    fn name(&self, id: FieldId) -> &Short {
        match &self.fields[id] {
            Field::Html { name, .. } | Field::Attribute { name, .. } => name,
        }
    }

    fn next_el(&mut self) -> Short {
        let n = self.elements;

        self.elements += 1;

        let mut var = Short::new();

        let _ = write!(var, "e{n}");

        var
    }
}

pub enum Field {
    Html {
        name: Short,
        value: TokenStream,
    },
    Attribute {
        name: Short,
        el: Short,
        abi: Short,
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
    Variable(Short),
    /// This node is an element that can be constructed in JavaScript
    JsNode(JsNode),
}

trait IntoGenerator {
    fn into_gen(self, gen: &mut Generator) -> DomNode;
}

impl IntoGenerator for Expression {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let name = gen.add_expression(self.stream);

        DomNode::Variable(*name)
    }
}

impl IntoGenerator for Node {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        match self {
            Node::Component(component) => component.into_gen(gen),
            Node::Expression(expr) => expr.into_gen(gen),
            Node::Text(lit) => DomNode::JsNode(JsNode::TextNode(JsString(lit))),
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
