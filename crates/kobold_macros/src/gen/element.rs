// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{Arguments, Write};

use tokens::{Literal, TokenStream};

use crate::dom::{Attribute, AttributeValue, CssValue, ElementTag, HtmlElement};
use crate::gen::{append, DomNode, Generator, IntoGenerator, JsArgument, Short};
use crate::itertools::IteratorExt as _;
use crate::parse::IteratorExt as _;
use crate::tokenize::prelude::*;

pub struct JsElement {
    /// Tag name of the element such as `div`
    pub tag: ElementTag,

    /// The `web-sys` type of this element, such as `HtmlElement`, spanned to tag invocation.
    pub typ: &'static str,

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
        let typ = element_js_type(&self.name);

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
                CssValue::Literal(class) => writeln!(el, "{var}.className={class};"),
                CssValue::Expression(expr) => {
                    el.hoisted = true;

                    let attr = Attr {
                        name: "ClassName",
                        abi: Some(InlineAbi::Str),
                    };
                    let class = gen
                        .add_field(expr.stream)
                        .attr(el.var, attr, attr.prop())
                        .name;

                    el.args.push(JsArgument::with_abi(class, InlineAbi::Str));

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

                let attr = Attr {
                    name: "Class",
                    abi: Some(InlineAbi::Str),
                };

                for class in self.classes {
                    if let CssValue::Expression(expr) = class {
                        el.hoisted = true;
                        let class = gen
                            .add_field(expr.stream)
                            .attr(el.var, attr, attr.prop())
                            .name;

                        el.args.push(JsArgument::with_abi(class, InlineAbi::Str));

                        writeln!(el, "{class} && {var}.classList.add({class});");
                    }
                }
            }
        }

        for Attribute { name, value } in self.attributes {
            let attr_type = attribute_type(&name.label);

            match value {
                AttributeValue::Literal(value) => {
                    writeln!(el, "{var}.setAttribute(\"{name}\",{value});");
                }
                AttributeValue::Boolean(value) => {
                    writeln!(el, "{var}.{name}={value};");
                }
                AttributeValue::Expression(mut expr) => match &attr_type {
                    AttributeType::Event(event) => {
                        let target = el.typ;

                        let coerce = if is_inline_closure(&mut expr.stream) {
                            call(
                                format_args!(
                                    "::kobold::internal::fn_type_hint::<\
                                    ::kobold::event::{event}<\
                                        ::kobold::reexport::web_sys::{target}\
                                    >,\
                                    _,\
                                >"
                                ),
                                expr.stream,
                            )
                        } else {
                            expr.stream
                        };

                        let value = gen.add_field(coerce).event(event, el.typ).name;

                        writeln!(
                            el,
                            "{var}.addEventListener(\"{}\",{value});",
                            &name.label[2..]
                        );

                        el.args.push(JsArgument::with_abi(value, InlineAbi::Event))
                    }
                    AttributeType::Provided(attr) => {
                        el.hoisted = true;

                        let value = gen
                            .add_field(expr.stream)
                            .attr(var, *attr, attr.prop())
                            .name;

                        if let Some(abi) = attr.abi {
                            writeln!(el, "{var}.{name}={value};");
                            el.args.push(JsArgument::with_abi(value, abi))
                        }
                    }
                    AttributeType::Unknown => {
                        el.hoisted = true;

                        let prop = (Literal::string(&name.label), ".into()").tokenize();
                        let attr = Attr::new("&AttributeName");

                        gen.add_field(expr.stream).attr(var, attr, prop);
                    }
                },
            };

            match attr_type {
                AttributeType::Event(event) => {
                    let target = el.typ;

                    gen.add_hint(
                        name.ident,
                        format_args!(
                            "impl Fn(\
                                ::kobold::event::{event}<\
                                    ::kobold::reexport::web_sys::{target}\
                                >\
                            ) + 'static"
                        ),
                    );
                }
                AttributeType::Provided(attr) => {
                    gen.add_attr_hint(name.ident, "", attr.name);
                }
                AttributeType::Unknown => {
                    gen.add_attr_hint(name.ident, "&'static", "AttributeName");
                }
            }
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
    Event,
}

impl InlineAbi {
    pub fn abi(self) -> &'static str {
        match self {
            InlineAbi::Bool => "bool",
            InlineAbi::Str => "&str",
            InlineAbi::Event => "wasm_bindgen::JsValue",
        }
    }

    pub fn method(self) -> Option<&'static str> {
        match self {
            InlineAbi::Bool => Some(".into()"),
            InlineAbi::Str => Some(".as_ref()"),
            InlineAbi::Event => None,
        }
    }

    pub fn bound(self) -> &'static str {
        match self {
            InlineAbi::Bool => "+ Into<bool> + Copy",
            InlineAbi::Str => "+ AsRef<str>",
            InlineAbi::Event => "",
        }
    }
}

#[derive(Clone, Copy)]
enum AttributeType {
    Provided(Attr),
    Event(&'static str),
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

    pub fn as_parts(&self) -> (&str, &str) {
        if self.name.starts_with('&') {
            ("&'static ", &self.name[1..])
        } else {
            ("", self.name)
        }
    }

    fn prop(&self) -> TokenStream {
        format_args!("::kobold::attribute::{}", self.name).tokenize()
    }
}

fn is_inline_closure(out: &mut TokenStream) -> bool {
    let mut is_closure = false;
    let mut stream = std::mem::replace(out, TokenStream::new()).parse_stream();

    if let Some(tt) = stream.allow_consume("move") {
        out.write(tt);
    }

    if let Some(tt) = stream.allow_consume('|') {
        is_closure = true;
        out.write(tt);
    }

    out.extend(stream);

    is_closure
}

fn attribute_type(attr: &str) -> AttributeType {
    if attr.starts_with("on") && attr.len() > 2 {
        return AttributeType::Event(event_js_type(&attr[2..]));
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
            name: "Value",
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
