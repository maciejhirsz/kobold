use std::fmt::{Arguments, Write};

use crate::dom::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen::{append, Abi, DomNode, Generator, IntoGenerator, JsArgument, Short};
use crate::itertools::IteratorExt;
use crate::parse::IdentExt;
use crate::tokenize::prelude::*;

pub struct JsElement {
    /// Tag name of the element such as `div`
    pub tag: String,

    /// Variable name of the element, such as `e0`
    pub var: Short,

    /// Method calls on constructed element, such as `e0.append(foo);` or `e0.className = bar;`
    pub code: String,

    /// Arguments to import from rust, with optional ABI type
    pub args: Vec<JsArgument>,

    /// Whether or not this element needs to be hoisted in its own JS function
    pub hoisted: bool,
}

impl JsElement {
    fn write_fmt(&mut self, args: Arguments) {
        let _ = self.code.write_fmt(args);
    }
}

impl IntoGenerator for HtmlElement {
    fn into_gen(mut self, gen: &mut Generator) -> DomNode {
        let var = gen.names.next_el();

        gen.set_js_type(element_js_type(&self.name));

        let mut el = JsElement {
            tag: self.name,
            var,
            code: String::new(),
            args: Vec::new(),
            hoisted: self.classes.iter().any(CssValue::is_expression),
        };

        match self.classes.len() {
            0 => (),
            1 => match self.classes.remove(0) {
                CssValue::Literal(class) => writeln!(el, "{var}.className={class};"),
                CssValue::Expression(expr) => {
                    let class = gen.add_attribute(
                        el.var,
                        Abi::Borrowed("&'abi str"),
                        call("::kobold::attribute::ClassName::from", expr.stream),
                    );

                    el.args.push(JsArgument::with_abi(class, "&str"));

                    writeln!(el, "{var}.className={class};");
                }
            },
            _ => {
                let lit_count = self.classes.iter().map(CssValue::is_literal).count();

                if lit_count > 0 {
                    let classes = self
                        .classes
                        .iter()
                        .filter_map(CssValue::as_literal)
                        .join(",");

                    writeln!(el, "{var}.classList.add({classes});");
                }

                for class in self.classes {
                    if let CssValue::Expression(expr) = class {
                        let class = gen.add_attribute(
                            el.var,
                            Abi::Borrowed("&'abi str"),
                            call("::kobold::attribute::Class::from", expr.stream),
                        );

                        el.args.push(JsArgument::with_abi(class, "&str"));

                        writeln!(el, "{class} && {var}.classList.add({class});");
                    }
                }
            }
        }

        for Attribute { name, value } in self.attributes {
            match value {
                AttributeValue::Literal(value) => {
                    writeln!(el, "{var}.setAttribute(\"{name}\",{value});");
                }
                AttributeValue::Boolean(value) => {
                    writeln!(el, "{var}.{name}={value};");
                }
                AttributeValue::Expression(expr) => {
                    let arg = match name.with_str(attribute_type) {
                        AttributeType::Event { event } => {
                            let target = element_js_type(el.tag.as_str());
                            let event_type = event_js_type(&event);

                            let callback = call(
                                format_args!(
                                    "::kobold::event::event_handler::<\
                                        ::kobold::event::{event_type}<\
                                            ::kobold::reexport::web_sys::{target}\
                                        >\
                                    >"
                                ),
                                expr.stream,
                            );

                            let value = gen.add_expression(callback);

                            writeln!(el, "{var}.addEventListener(\"{event}\",{value});");

                            JsArgument::new(value)
                        }
                        AttributeType::Provided { attr_type } => {
                            el.hoisted = true;
                            unimplemented!()
                        }
                        AttributeType::Unknown => {
                            el.hoisted = true;
                            unimplemented!()
                        }
                    };
                    // } else if attr == "checked" {
                    //     el.hoisted = true;
                    //     let value = gen.add_attribute(
                    //         var,
                    //         Abi::Owned("bool"),
                    //         call("::kobold::attribute::Checked", expr.stream),
                    //     );

                    //     writeln!(el, "{var}.{attr}={value};");

                    //     JsArgument::with_abi(value, "bool")
                    // } else if provided_attr(attr) {
                    //     let value = gen.add_expression(call(
                    //         format_args!("::kobold::attribute::{attr}"),
                    //         expr.stream,
                    //     ));

                    //     writeln!(el, "{var}.setAttributeNode({value});");

                    //     JsArgument::new(value)
                    // } else {
                    //     let attr_fn = gen.attribute_constructor(attr);

                    //     let value = gen.add_expression(call(
                    //         "::kobold::attribute::AttributeNode::new",
                    //         (ident(&attr_fn), ',', expr.stream),
                    //     ));

                    //     writeln!(el, "{var}.setAttributeNode({value});");

                    //     JsArgument::new(value)
                    // };

                    el.args.push(arg);
                }
            };
        }

        if let Some(children) = self.children {
            let append = append(gen, &mut el.code, &mut el.args, children);
            writeln!(el, "{var}.{append};");
        }

        DomNode::Element(el)
    }
}

enum AttributeType {
    Provided {
        // inline: Option<&'static str>,
        attr_type: &'static str,
    },
    Event {
        event: Box<str>
    },
    Unknown,
}

fn attribute_type(attr: &str) -> AttributeType {
    if attr.starts_with("on") && attr.len() > 2 {
        return AttributeType::Event { event: attr[2..].into() };
    }

    match attr {
        "href" => AttributeType::Provided {
            attr_type: "Href",
        },
        "style" => AttributeType::Provided {
            attr_type: "Style",
        },
        "value" => AttributeType::Provided {
            attr_type: "InputValue",
        },
        "checked" => AttributeType::Provided {
            attr_type: "Checked",
        },
        _ => AttributeType::Unknown,
    }
}

#[rustfmt::skip]
fn provided_attr(attr: &str) -> bool {
    match attr {
        "href"
        | "style"
        | "value" => true,
        _ => false,
    }
}

#[rustfmt::skip]
fn event_js_type(event: &str) -> &'static str {
    match event {
        "click"
        | "dblclick"
        | "mousedown"
        | "mouseup"
        | "mouseover"
        | "mousemove"
        | "mouseout"
        | "mouseenter"
        | "mouseleave" => "MouseEvent",

        "keydown"
        | "keyup"
        | "keypress" => "KeyboardEvent",
        _ => "Event",
    }
}

fn element_js_type(tag: &str) -> &'static str {
    match tag {
        "a" => "HtmlLinkElement",
        "canvas" => "HtmlCanvasElement",
        "form" => "HtmlFormElement",
        "img" => "HtmlImageElement",
        "input" => "HtmlInputElement",
        "option" => "HtmlOptionElement",
        "select" => "HtmlSelectElement",
        "textarea" => "HtmlTextAreaElement",
        _ => "HtmlElement",
    }
}
