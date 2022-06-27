use std::fmt::{Display, Write};

use crate::dom2::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen2::{DomNode, FieldId, Generator, IntoGenerator};

pub struct JsElement {
    /// Constructor for the element, such as `document.createElement('div')`
    ///
    /// NOTE: no semicolon
    pub constructor: String,

    /// Method calls on constructed element, such as `append(foo)` or `className = bar`
    ///
    /// NOTE: no dot, no semicolon
    pub calls: Vec<String>,

    /// Arguments from Rust this snippet requires
    pub args: Vec<FieldId>,
}

impl JsElement {
    fn text_node(lit: proc_macro::Literal) -> Self {
        JsElement {
            constructor: format!("document.createTextNode({lit})"),
            calls: Vec::new(),
            args: Vec::new(),
        }
    }
}

fn into_class_name<'a>(
    class: &'a mut Option<CssValue>,
    args: &mut Vec<FieldId>,
    gen: &'a mut Generator,
) -> Option<&'a dyn Display> {
    if let Some(CssValue::Literal(lit)) = class {
        return Some(lit);
    }

    if let Some(CssValue::Expression(expr)) = class.take() {
        let id = expr.into_var(gen);

        args.push(id);

        return Some(gen.out.name(id));
    }

    None
}

impl IntoGenerator for HtmlElement {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let constructor = format!("document.createElement('{}')", self.name);
        let count = self.classes.len();

        let mut calls = Vec::new();
        let mut args = Vec::new();

        let mut classes = self.classes.into_iter();

        if count == 0 {
            if let Some(class) = into_class_name(&mut classes.next(), &mut args, gen) {
                calls.push(format!("className={class}"));
            }
        } else if let Some(first) = into_class_name(&mut classes.next(), &mut args, gen) {
            let mut class_list = format!("classList.add({first}");

            while let Some(class) = into_class_name(&mut classes.next(), &mut args, gen) {
                let _ = write!(&mut class_list, ",{class}");
            }

            class_list.push(')');

            calls.push(class_list);
        }

        for Attribute { name, value } in self.attributes {
            match value {
                AttributeValue::Literal(value) => {
                    calls.push(format!("setAttribute('{name}',{value})"));
                }
                AttributeValue::Boolean(value) => calls.push(format!("{name}={value}")),
                AttributeValue::Expression(expr) => {
                    let id = expr.into_var(gen);

                    args.push(id);

                    let value = gen.out.name(id);

                    calls.push(format!("setAttributeNode('{name}',{value}"));
                }
            };
        }

        // TODO: attributes

        if let Some(children) = self.children {
            let mut append = String::from("append(");

            for child in children {
                let dom_node = child.into_gen(gen);

                let value: &dyn Display = match &dom_node {
                    DomNode::TextNode(text) => text,
                    DomNode::Variable(id) => {
                        args.push(*id);

                        gen.out.name(*id)
                    }
                    DomNode::Element(_) => unimplemented!(),
                };

                let _ = write!(&mut append, "{value},");
            }

            append.pop();
            append.push(')');

            calls.push(append);
        }

        DomNode::Element(JsElement {
            constructor,
            calls,
            args,
        })
    }
}
