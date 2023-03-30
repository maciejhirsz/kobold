// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{Arguments, Write};

use proc_macro::{Ident, Literal, TokenStream};

use crate::dom::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen::{append, DomNode, Generator, IntoGenerator, JsArgument, Short};
use crate::itertools::IteratorExt;
use crate::parse::IdentExt;
use crate::tokenize::prelude::*;

pub struct JsElement {
    /// Tag name of the element such as `div`
    pub tag: String,

    /// The `web-sys` type of this element, such as `HtmlElement`, spanned to tag invocation.
    pub typ: Ident,

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
        let typ = Ident::new(element_js_type(&self.name), self.span);

        let mut el = JsElement {
            tag: self.name,
            typ,
            var,
            code: String::new(),
            args: Vec::new(),
            hoisted: false, // None, // self.classes.iter().any(CssValue::is_expression),
        };

        match self.classes.len() {
            0 => (),
            1 => match self.classes.remove(0) {
                (_, CssValue::Literal(class)) => writeln!(el, "{var}.className={class};"),
                (span, CssValue::Expression(expr)) => {
                    el.hoisted = true;

                    let attr = Attr {
                        name: "ClassName",
                        abi: Some(InlineAbi::Str),
                    };
                    let name = Ident::new("class", span);

                    let class = gen
                        .add_field(expr.stream)
                        .attr(el.var, name, attr, attr.prop())
                        .name;

                    el.args.push(JsArgument::with_abi(class, InlineAbi::Str));

                    writeln!(el, "{var}.className={class};");
                }
            },
            _ => {
                let lit_count = self.classes.iter().map(|v| v.1.is_literal()).count();

                if lit_count > 0 {
                    let classes = self
                        .classes
                        .iter()
                        .filter_map(|v| v.1.as_literal())
                        .join(",");

                    writeln!(el, "{var}.classList.add({classes});");
                }

                let attr = Attr {
                    name: "Class",
                    abi: Some(InlineAbi::Str),
                };

                for (i, class) in self.classes.into_iter().enumerate() {
                    if let (span, CssValue::Expression(expr)) = class {
                        el.hoisted = true;
                        let name = Ident::new(&format!("class_{i}"), span);

                        let class = gen
                            .add_field(expr.stream)
                            .attr(el.var, name, attr, attr.prop())
                            .name;

                        el.args.push(JsArgument::with_abi(class, InlineAbi::Str));

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
                AttributeValue::Expression(expr) => match name.with_str(attribute_type) {
                    AttributeType::Event { event } => {
                        let event_type = event_js_type(&event);
                        let target = el.typ.clone();

                        let coerce = block((
                            call(
                                "fn coerce",
                                (
                                    name.clone(),
                                    format_args!(
                                        ": impl Fn(::kobold::event::{event_type}<\
                                            ::kobold::reexport::web_sys::{target}\
                                        >) + 'static"
                                    ),
                                ),
                            ),
                            format_args!(
                                " -> impl ::kobold::event::Listener<\
                                    ::kobold::event::{event_type}<\
                                        ::kobold::reexport::web_sys::{target}\
                                    >\
                                >"
                            ),
                            block(name.tokenize()),
                            call("coerce", expr.stream),
                        ))
                        .tokenize();

                        let value = gen.add_field(coerce).event(event_type, target).name;

                        writeln!(el, "{var}.addEventListener(\"{event}\",{value});");

                        el.args.push(JsArgument::new(value))
                    }
                    AttributeType::Provided(attr) => {
                        el.hoisted = true;

                        let value = gen
                            .add_field(expr.stream)
                            .attr(var, name.clone(), attr, attr.prop())
                            .name;

                        if let Some(abi) = attr.abi {
                            writeln!(el, "{var}.{name}={value};");
                            el.args.push(JsArgument::with_abi(value, abi))
                        }
                    }
                    AttributeType::Unknown => {
                        el.hoisted = true;

                        let prop = name.with_str(Literal::string).tokenize();
                        let attr = Attr::new("Attribute");
                        gen.add_field(expr.stream).attr(var, name, attr, prop);
                    }
                },
            };
        }

        if let Some(children) = self.children {
            let append = append(gen, &mut el.code, &mut el.args, children);
            writeln!(el, "{var}.{append};");
        }

        DomNode::Element(el)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InlineAbi {
    Bool,
    Str,
}

impl InlineAbi {
    pub fn abi(self) -> &'static str {
        match self {
            InlineAbi::Bool => "bool",
            InlineAbi::Str => "&str",
        }
    }

    pub fn method(self) -> &'static str {
        match self {
            InlineAbi::Bool => ".into()",
            InlineAbi::Str => ".as_ref()",
        }
    }

    pub fn bound(self) -> &'static str {
        match self {
            InlineAbi::Bool => "+ Into<bool> + Copy",
            InlineAbi::Str => "+ AsRef<str>",
        }
    }
}

enum AttributeType {
    Provided(Attr),
    Event { event: Box<str> },
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub struct Attr {
    pub name: &'static str,
    pub abi: Option<InlineAbi>,
}

impl Attr {
    const fn new(name: &'static str) -> Self {
        Attr { name, abi: None }
    }

    fn prop(&self) -> TokenStream {
        format_args!("::kobold::attribute::{}", self.name).tokenize()
    }
}

fn attribute_type(attr: &str) -> AttributeType {
    if attr.starts_with("on") && attr.len() > 2 {
        return AttributeType::Event {
            event: attr[2..].into(),
        };
    }

    let attr = match attr {
        "checked" => Attr {
            name: "Checked",
            abi: Some(InlineAbi::Bool),
        },
        "href" => Attr {
            name: "Href",
            abi: Some(InlineAbi::Str),
        },
        "style" => Attr {
            name: "Style",
            abi: Some(InlineAbi::Str),
        },
        "value" => Attr {
            name: "InputValue",
            abi: None,
        },
        _ => return AttributeType::Unknown,
    };

    AttributeType::Provided(attr)
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
