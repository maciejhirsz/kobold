use std::fmt::{self, Arguments, Display, Write};

use proc_macro::Literal;

use crate::dom2::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen2::{append, DomNode, Generator, IntoGenerator, JsArgument, Short, TokenStreamExt};
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

fn into_class_name(
    class: Option<CssValue>,
    el: &mut JsElement,
    gen: &mut Generator,
) -> Option<ClassName> {
    match class? {
        CssValue::Literal(lit) => Some(ClassName::Literal(lit)),
        CssValue::Expression(expr) => {
            let expr = ("::kobold::attribute::Class", group('(', expr.stream)).tokenize();

            let name = gen.add_attribute(el.var, "&'static str", expr);

            el.args.push(JsArgument::with_abi(name, "&str"));

            Some(ClassName::Expression(name))
        }
    }
}

enum ClassName {
    Literal(Literal),
    Expression(Short),
}

impl Display for ClassName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClassName::Literal(lit) => lit.fmt(f),
            ClassName::Expression(short) => f.write_str(short),
        }
    }
}

impl IntoGenerator for HtmlElement {
    fn into_gen(self, gen: &mut Generator) -> DomNode {
        let var = gen.names.next_el();

        let mut el = JsElement {
            tag: self.name,
            var,
            code: String::new(),
            args: Vec::new(),
            hoisted: self.classes.iter().any(CssValue::is_expression),
        };

        let mut classes = self.classes.into_iter();

        if classes.len() == 1 {
            if let Some(class) = into_class_name(classes.next(), &mut el, gen) {
                writeln!(el, "{var}.className={class};");
            }
        } else if let Some(first) = into_class_name(classes.next(), &mut el, gen) {
            write!(el, "{var}.classList.add({first}");

            while let Some(class) = into_class_name(classes.next(), &mut el, gen) {
                write!(el, ",{class}");
            }

            el.code.push_str(");\n");
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
                        let target = match el.tag.as_str() {
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
                            (
                                bind.invocation,
                                format_args!("::<::kobold::reexport::web_sys::{target}, _, _> ="),
                                bind.arg,
                            )
                                .tokenize()
                        } else {
                            group(
                                '{',
                                (
                                    format_args!(
                                        "let constrained: ::kobold::stateful::Callback<\
                                            ::kobold::reexport::web_sys::{target},\
                                            _,\
                                            _,\
                                        > ="
                                    ),
                                    expr.stream,
                                    "; constrained",
                                ),
                            )
                            .tokenize()
                        };

                        let event = &attr[2..];
                        let value = gen.add_expression(callback);

                        writeln!(el, "{var}.addEventListener(\"{event}\",{value});");

                        JsArgument::new(value)
                    } else if attr == "checked" {
                        el.hoisted = true;

                        let value = gen.add_attribute(var, "bool", expr.stream);

                        writeln!(el, "{var}.{attr}={value};");

                        JsArgument::with_abi(value, "bool")
                    } else {
                        let expr = (
                            "::kobold::attribute::AttributeNode::new",
                            group('(', (string(attr), ',', expr.stream)),
                        );

                        let value = gen.add_expression(expr.tokenize());

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
