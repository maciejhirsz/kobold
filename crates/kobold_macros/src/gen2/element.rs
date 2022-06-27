use std::fmt::{Display, Write};
use std::str::FromStr;

use crate::dom2::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen2::{DomNode, Field, FieldId, Generator, IntoGenerator, JsString};
use crate::parse::{IdentExt, Parse, TokenStreamExt};
use crate::syntax::InlineBind;

pub enum JsNode {
    TextNode {
        /// Contents of the TextNode
        text: JsString,
    },
    Element {
        /// Tag name of the element such as `div`
        tag: String,

        /// Method calls on constructed element, such as `append(foo)` or `className = bar`
        ///
        /// NOTE: no dot, no semicolon
        calls: Vec<String>,

        /// Arguments from Rust this snippet requires
        args: Vec<FieldId>,
    },
    Fragment {
        /// Children belonging to this document fragment
        chidren: Vec<JsNode>,
    },
}

impl JsNode {
    fn text_node(lit: proc_macro::Literal) -> Self {
        JsNode::TextNode {
            text: JsString(lit),
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
        let id = gen.add_expression(expr.stream.into());

        args.push(id);

        return Some(gen.out.name(id));
    }

    None
}

impl IntoGenerator for HtmlElement {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let tag = self.name.to_string();
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
                AttributeValue::Expression(expr) => name.with_str(|name| {
                    if name.starts_with("on") && name.len() > 2 {
                        let target = match tag.as_str() {
                            "a" => "HtmlLinkElement",
                            "form" => "HtmlFormElement",
                            "img" => "HtmlImageElement",
                            "input" => "HtmlInputElement",
                            "option" => "HtmlOptionElement",
                            "select" => "HtmlSelectElement",
                            "textarea" => "HtmlTextAreaElement",
                            _ => "HtmlElement",
                        };

                        let mut inner = expr.stream.clone().parse_stream();

                        let expr = if let Ok(bind) = InlineBind::parse(&mut inner) {
                            let mut expr = bind.invocation;
                            expr.write(&format!(
                                "::<::kobold::reexport::web_sys::{target}, _, _> ="
                            ));
                            expr.push(bind.arg);
                            expr
                        } else {
                            use proc_macro::{Delimiter, TokenStream};

                            let mut con = TokenStream::from_str(&format!(
                                "let constrained: ::kobold:stateful::Callback<\
                                    ::kobold::reexport::web_sys::{target},\
                                    _,\
                                    _,\
                                > ="
                            ))
                            .unwrap();
                            con.extend(expr.stream);
                            con.write("; constrained");
                            con.group(Delimiter::Brace)
                        };

                        let (typ, name) = gen.names.next();

                        // TODO: Do something with this expression!
                        gen.add_expression(expr.into());

                        return;
                    }

                    let id = gen.add_expression(expr.stream.into());

                    args.push(id);

                    let value = gen.out.name(id);

                    calls.push(format!("setAttributeNode('{name}',{value}"));
                }),
            };
        }

        if let Some(children) = self.children {
            let mut append = String::from("append(");

            for child in children {
                let dom_node = child.into_gen(gen);

                let value: &dyn Display = match &dom_node {
                    DomNode::Variable(id) => {
                        args.push(*id);

                        gen.out.name(*id)
                    }
                    DomNode::JsNode(JsNode::TextNode { text }) => text,
                    DomNode::JsNode(_) => unimplemented!(),
                };

                let _ = write!(&mut append, "{value},");
            }

            append.pop();
            append.push(')');

            calls.push(append);
        }

        DomNode::JsNode(JsNode::Element { tag, calls, args })
    }
}
