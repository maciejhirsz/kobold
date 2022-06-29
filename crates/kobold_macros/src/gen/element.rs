use std::fmt::{Arguments, Write};

use proc_macro::TokenStream;

use crate::dom::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen::{
    append, Abi, DomNode, Generator, IntoGenerator, JsArgument, Short, TokenStreamExt,
};
use crate::itertools::IteratorExt;
use crate::parse::{IdentExt, Parse};
use crate::syntax::InlineBind;
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
                        call("::kobold::attribute::ClassName", expr.stream),
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
                            call("::kobold::attribute::Class", expr.stream),
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
                AttributeValue::Expression(expr) => name.with_str(|attr| {
                    let arg = if attr.starts_with("on") && attr.len() > 2 {
                        let target = element_js_type(el.tag.as_str());

                        let mut inner = expr.stream.clone().parse_stream();

                        let callback = if let Ok(bind) = InlineBind::parse(&mut inner) {
                            (
                                bind.invocation,
                                format_args!("::<::kobold::reexport::web_sys::{target}, _, _>"),
                                bind.arg,
                            )
                                .tokenize()
                        } else {
                            block((
                                format_args!(
                                    "let constrained: ::kobold::stateful::Callback<\
                                        ::kobold::reexport::web_sys::{target},\
                                        _,\
                                        _,\
                                    > ="
                                ),
                                expr.stream,
                                "; constrained",
                            ))
                            .tokenize()
                        };

                        let event = &attr[2..];
                        let value = gen.add_expression(callback);

                        writeln!(el, "{var}.addEventListener(\"{event}\",{value});");

                        JsArgument::new(value)
                    } else if attr == "checked" {
                        el.hoisted = true;
                        let value = gen.add_attribute(
                            var,
                            Abi::Owned("bool"),
                            call("::kobold::attribute::Checked", expr.stream),
                        );

                        writeln!(el, "{var}.{attr}={value};");

                        JsArgument::with_abi(value, "bool")
                    } else {
                        let value = gen.add_expression(call(
                            "::kobold::attribute::AttributeNode::new",
                            (string(attr), ',', expr.stream),
                        ));

                        writeln!(el, "{var}.setAttributeNode({value});");

                        JsArgument::new(value)
                    };

                    el.args.push(arg);
                }),
            };
        }

        if let Some(children) = self.children {
            let append = append(gen, &mut el.code, &mut el.args, children);
            writeln!(el, "{var}.{append};");
        }

        DomNode::Element(el)
    }
}

fn element_js_type(tag: &str) -> &'static str {
    match tag {
        "a" => "HtmlLinkElement",
        "form" => "HtmlFormElement",
        "img" => "HtmlImageElement",
        "input" => "HtmlInputElement",
        "option" => "HtmlOptionElement",
        "select" => "HtmlSelectElement",
        "textarea" => "HtmlTextAreaElement",
        _ => "HtmlElement",
    }
}
