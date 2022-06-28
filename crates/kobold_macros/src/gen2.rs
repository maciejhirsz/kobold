use std::fmt::{self, Debug, Display, Write};
use std::hash::Hash;

use arrayvec::ArrayString;
use proc_macro::{Literal, TokenStream};

use crate::dom2::{Expression, Node};

mod component;
mod element;
mod fragment;
mod token_stream;

pub use element::JsElement;
pub use fragment::{append, JsFragment};
pub use token_stream::TokenStreamExt;

// Short string for auto-generated variable names
pub type Short = ArrayString<4>;

// JS function name, capacity must fit a `Short`, a hash, and few underscores
pub type JsFnName = ArrayString<24>;

pub enum DomNode {
    Variable(Short),
    TextNode(JsString),
    Element(JsElement),
    Fragment(JsFragment),
}

impl DomNode {
    pub fn text_node(lit: Literal) -> Self {
        DomNode::TextNode(JsString(lit))
    }
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
    fn add_expression(&mut self, value: TokenStream) -> Short {
        let name = self.names.next();

        self.out.fields.push(Field::Html { name, value });

        name
    }

    fn add_attribute(&mut self, el: Short, abi: &'static str, value: TokenStream) -> Short {
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

        let (var, body, args) = match node {
            DomNode::Variable(_) => return None,
            DomNode::TextNode(text) => {
                let body = format!("return document.createTextNode({text});\n");
                let var = self.names.next_el();

                (var, body, Vec::new())
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

                (var, body, args)
            }
            DomNode::Fragment(JsFragment { var, code, args }) => {
                assert!(
                    !code.is_empty(),
                    "Document fragment mustn't be empty, this is a bug"
                );

                (var, code, args)
            }
        };

        self.out.els.push(var);

        let mut hasher = fnv::FnvHasher::default();
        var.hash(&mut hasher);
        body.hash(&mut hasher);

        let hash = hasher.finish();
        let name = JsFnName::try_from(format_args!("__{var}_{hash:016x}")).unwrap();

        let js = &mut self.out.js.code;

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

        self.out.js.functions.push(JsFunction { name, args });

        Some(name)
    }
}

#[derive(Default)]
pub struct JsModule {
    pub functions: Vec<JsFunction>,
    pub code: String,
}

impl Debug for JsModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.functions, f)?;

        write!(f, "\ncode: ({})", self.code)
    }
}

#[derive(Debug)]
pub struct JsFunction {
    pub name: JsFnName,
    pub args: Vec<Short>,
}

#[derive(Default, Debug)]
pub struct Transient {
    js: JsModule,
    fields: Vec<Field>,
    els: Vec<Short>,
}

pub enum Field {
    Html {
        name: Short,
        value: TokenStream,
    },
    Attribute {
        name: Short,
        el: Short,
        abi: &'static str,
        value: TokenStream,
    },
}

impl Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Field::Html { name, value } => {
                write!(f, "{name} <Html>: {value}")
            }
            Field::Attribute {
                name,
                el,
                abi,
                value,
            } => {
                write!(f, "{name} <Attribute({abi} -> {el})>: {value}")
            }
        }
    }
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

pub struct JsString(Literal);

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
