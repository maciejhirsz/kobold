use std::fmt::{Display, Write};

use crate::dom2::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen2::{DomNode, FieldId, Generator, IntoGenerator, JsString};
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

        /// Variable name of the element, such as `e0`
        var: usize,

        /// Method calls on constructed element, such as `e0.append(foo);` or `e0.className = bar;`
        code: String,

        /// Arguments from Rust this snippet requires
        args: Vec<FieldId>,
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
        let (id, name) = gen.add_expression(expr.stream);

        args.push(id);

        return Some(name);
    }

    None
}

impl IntoGenerator for HtmlElement {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let tag = self.name.to_string();
        let var = gen.out.next_el();
        let count = self.classes.len();

        let mut code = String::new();
        let mut args = Vec::new();

        let mut classes = self.classes.into_iter();

        let js = &mut code;

        if count == 0 {
            if let Some(class) = into_class_name(&mut classes.next(), &mut args, gen) {
                let _ = write!(js, "e{var}.className={class};");
            }
        } else if let Some(first) = into_class_name(&mut classes.next(), &mut args, gen) {
            let _ = write!(js, "e{var}.classList.add({first}");

            while let Some(class) = into_class_name(&mut classes.next(), &mut args, gen) {
                let _ = write!(js, ",{class}");
            }

            js.push_str(");");
        }

        for Attribute { name, value } in self.attributes {
            match value {
                AttributeValue::Literal(value) => {
                    let _ = write!(js, "e{var}.setAttribute('{name}',{value});");
                }
                AttributeValue::Boolean(value) => {
                    let _ = write!(js, "e{var}.{name}={value};");
                }
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

                        let callback = if let Ok(bind) = InlineBind::parse(&mut inner) {
                            let mut expr = bind.invocation;
                            write!(
                                expr,
                                "::<::kobold::reexport::web_sys::{target}, _, _> ="
                            );
                            expr.push(bind.arg);
                            expr
                        } else {
                            use proc_macro::{Delimiter, TokenStream};

                            let mut con = TokenStream::new();

                            write!(
                                con,
                                "let constrained: ::kobold:stateful::Callback<\
                                    ::kobold::reexport::web_sys::{target},\
                                    _,\
                                    _,\
                                > ="
                            );
                            con.extend(expr.stream);
                            con.write("; constrained");
                            con.group(Delimiter::Brace)
                        };

                        let event = &name[2..];
                        let (id, value) = gen.add_expression(callback);

                        args.push(id);

                        let _ = write!(js, "e{var}.addEventListener({event:?},{value});");

                        return;
                    }

                    let (id, value) = gen.add_expression(expr.stream);

                    args.push(id);

                    let _ = write!(js, "e{var}.setAttributeNode('{name}',{value});");
                }),
            };
        }

        if let Some(children) = self.children {
            let _ = write!(js, "e{var}.append(");

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

                let _ = write!(js, "{value},");
            }

            js.pop();
            js.push_str(");");
        }

        DomNode::JsNode(JsNode::Element {
            tag,
            var,
            code,
            args,
        })
    }
}
