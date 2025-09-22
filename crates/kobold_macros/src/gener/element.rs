// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{Arguments, Write};

use tokens::{Literal, TokenStream};

use crate::dom::{Attribute, AttributeValue, CssValue, ElementTag, Event, HtmlElement};
use crate::event::EventKind;
use crate::gener::{DomNode, Generator, IntoGenerator, JsArgument, Short, append};
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
    fn into_generator(mut self, gener: &mut Generator) -> DomNode {
        let var = gener.names.next_el();
        let typ = element_js_type(&self.name);

        let mut el = JsElement {
            tag: self.name,
            typ,
            var,
            code: String::new(),
            args: Vec::new(),
            hoisted: false,
        };

        match (self.classes.len(), el.tag.namespace().is_none()) {
            (0, _) => (),
            (1, true) => match self.classes.remove(0) {
                CssValue::Literal(class) => writeln!(el, "{var}.className={class};"),
                CssValue::Expression(expr) => {
                    el.hoisted = true;

                    let attr = Attr {
                        hint: "ClassName",
                        abi: None,
                    };
                    gener.add_field(expr.stream).attr(el.var, attr, attr.prop());
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
                    hint: "Class",
                    abi: None,
                };

                for class in self.classes {
                    if let CssValue::Expression(expr) = class {
                        el.hoisted = true;
                        gener.add_field(expr.stream).attr(el.var, attr, attr.prop());
                    }
                }
            }
        }

        for event in self.events {
            let Event {
                name,
                kind,
                mut value,
            } = event;

            let target = el.typ;
            let event = event_js_type(&name);

            let coerce = if is_inline_closure(&mut value.stream) {
                call(
                    format_args!(
                        "::kobold::internal::fn_type_hint::<\
                        ::kobold::event::{event}<\
                            ::kobold::reexport::web_sys::{target}\
                        >,\
                        _,\
                    >"
                    ),
                    value.stream,
                )
            } else {
                value.stream
            };

            let value = gener.add_field(coerce).event(event, el.typ).name;

            writeln!(el, "{var}[$_koboldSym[{}]]={value};", kind as usize);

            el.args.push(JsArgument::with_abi(value, InlineAbi::Event));

            gener.add_hint(
                name.ident,
                format_args!(
                    "impl Fn(\
                        &::kobold::event::{event}<\
                            ::kobold::reexport::web_sys::{target}\
                        >\
                    ) + 'static"
                ),
            );
        }

        for Attribute { name, value } in self.attributes {
            let handler = attribute_handler(&name);

            match value {
                AttributeValue::Literal(value) => match handler {
                    AttributeHandler::Link => {
                        writeln!(el, "{var}.href={value};");
                        writeln!(el, "{var}[$_koboldSym[{}]]=0;", EventKind::Click as usize);
                    }
                    AttributeHandler::Prop { name, .. } => {
                        writeln!(el, "{var}.{name}={value};");
                    }
                    AttributeHandler::SetAttribute { name } => {
                        writeln!(el, "{var}.setAttribute(\"{name}\",{value});");
                    }
                },
                AttributeValue::Boolean(value) => {
                    writeln!(el, "{var}.{name}={value};");
                }
                AttributeValue::Expression(expr) => match &handler {
                    AttributeHandler::Link => {
                        el.hoisted = true;

                        let attr = Attr {
                            hint: "Href",
                            abi: Some(InlineAbi::Str),
                        };

                        let value = gener
                            .add_field(expr.stream)
                            .attr(var, attr, attr.prop())
                            .name;

                        writeln!(el, "{var}.href={value};");
                        writeln!(el, "{var}[$_koboldSym[{}]]=0;", EventKind::Click as usize);
                    }
                    AttributeHandler::Prop { name, attr } => {
                        el.hoisted = true;

                        let value = gener
                            .add_field(expr.stream)
                            .attr(var, *attr, attr.prop())
                            .name;

                        if let Some(abi) = attr.abi {
                            writeln!(el, "{var}.{name}={value};");
                            el.args.push(JsArgument::with_abi(value, abi))
                        }
                    }
                    AttributeHandler::SetAttribute { name } => {
                        el.hoisted = true;

                        let prop = (Literal::string(name), ".into()").tokenize();
                        let attr = Attr::new("&AttributeName");

                        gener.add_field(expr.stream).attr(var, attr, prop);
                    }
                },
            };

            match handler {
                AttributeHandler::Prop { attr, .. } => {
                    gener.add_attr_hint(name.ident, "", attr.hint);
                }
                AttributeHandler::Link => {
                    gener.add_attr_hint(name.ident, "", "Href");
                }
                AttributeHandler::SetAttribute { .. } => {
                    gener.add_attr_hint(name.ident, "&'static", "AttributeName");
                }
            }
        }

        if let Some(children) = self.children {
            let append = append(gener, &mut el.code, &mut el.args, children);
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
            InlineAbi::Event => "u32",
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

#[derive(Debug, Clone, Copy)]
pub struct Attr {
    pub hint: &'static str,
    pub abi: Option<InlineAbi>,
}

impl Attr {
    const fn new(hint: &'static str) -> Self {
        Attr { hint, abi: None }
    }

    pub fn as_parts(&self) -> (&str, &str) {
        if self.hint.starts_with('&') {
            ("&'static ", &self.hint[1..])
        } else {
            ("", self.hint)
        }
    }

    fn prop(&self) -> TokenStream {
        format_args!("::kobold::attribute::{}", self.hint).tokenize()
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

#[derive(Clone, Copy)]
enum AttributeHandler<'name> {
    /// Use default `el.setAttribute(name, value);`
    SetAttribute { name: &'name str },
    /// Use as a prop `el.name = value;`
    Prop { name: &'name str, attr: Attr },
    /// This is a link and needs its own special case
    Link,
}

fn attribute_handler(name: &str) -> AttributeHandler<'_> {
    match name {
        "link" => AttributeHandler::Link,
        "view_box" => AttributeHandler::SetAttribute { name: "viewBox" },
        "checked" => AttributeHandler::Prop {
            name,
            attr: Attr {
                hint: "Checked",
                abi: Some(InlineAbi::Bool),
            },
        },
        "href" => AttributeHandler::Prop {
            name,
            attr: Attr {
                hint: "Href",
                abi: Some(InlineAbi::Str),
            },
        },
        "html" => AttributeHandler::Prop {
            name: "innerHTML",
            attr: Attr {
                hint: "InnerHtml",
                abi: Some(InlineAbi::Str),
            },
        },
        "style" => AttributeHandler::Prop {
            name,
            attr: Attr {
                hint: "Style",
                abi: Some(InlineAbi::Str),
            },
        },
        "value" => AttributeHandler::Prop {
            name,
            attr: Attr {
                hint: "Value",
                abi: None,
            },
        },
        name => AttributeHandler::SetAttribute { name },
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
