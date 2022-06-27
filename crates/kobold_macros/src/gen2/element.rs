use std::fmt::{self, Display, Write};

use proc_macro::{Delimiter, Group, Literal, TokenStream};

use crate::dom2::{Attribute, AttributeValue, CssValue, HtmlElement};
use crate::gen2::{append, DomNode, Generator, IntoGenerator, Short};
use crate::parse::{IdentExt, Parse, TokenStreamExt};
use crate::syntax::InlineBind;

pub struct JsElement {
    /// Tag name of the element such as `div`
    pub tag: String,

    /// Variable name of the element, such as `e0`
    pub var: Short,

    /// Method calls on constructed element, such as `e0.append(foo);` or `e0.className = bar;`
    pub code: String,

    /// Arguments to import from rust
    pub args: Vec<Short>,

    /// Whether or not this element needs to be hoisted in its own JS function
    pub hoisted: bool,
}

fn into_class_name(
    class: Option<CssValue>,
    el: &mut JsElement,
    gen: &mut Generator,
) -> Option<ClassName> {
    match class? {
        CssValue::Literal(lit) => Some(ClassName::Literal(lit)),
        CssValue::Expression(expr) => {
            let name = gen.add_attribute(el.var, "&str", expr.stream);

            el.args.push(name);

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
                let _ = writeln!(el.code, "{var}.className={class};");
            }
        } else if let Some(first) = into_class_name(classes.next(), &mut el, gen) {
            let _ = write!(el.code, "{var}.classList.add({first}");

            while let Some(class) = into_class_name(classes.next(), &mut el, gen) {
                let _ = write!(el.code, ",{class}");
            }

            el.code.push_str(");\n");
        }

        for Attribute { name, value } in self.attributes {
            match value {
                AttributeValue::Literal(value) => {
                    let _ = writeln!(el.code, "{var}.setAttribute(\"{name}\",{value});");
                }
                AttributeValue::Boolean(value) => {
                    let _ = writeln!(el.code, "{var}.{name}={value};");
                }
                AttributeValue::Expression(expr) => name.with_str(|attr| {
                    let value = if attr.starts_with("on") && attr.len() > 2 {
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
                            let mut expr = bind.invocation;
                            write!(expr, "::<::kobold::reexport::web_sys::{target}, _, _> =");
                            expr.push(bind.arg);
                            expr
                        } else {
                            let mut constrain = TokenStream::new();

                            write!(
                                constrain,
                                "let constrained: ::kobold:stateful::Callback<\
                                    ::kobold::reexport::web_sys::{target},\
                                    _,\
                                    _,\
                                > ="
                            );
                            constrain.extend(expr.stream);
                            constrain.write("; constrained");
                            constrain.group(Delimiter::Brace)
                        };

                        let event = &attr[2..];
                        let value = gen.add_expression(callback);

                        let _ = writeln!(el.code, "{var}.addEventListener(\"{event}\",{value});");

                        value
                    } else if attr == "checked" {
                        el.hoisted = true;

                        let value = gen.add_attribute(var, "bool", expr.stream);

                        let _ = writeln!(el.code, "{var}.{attr}={value};");

                        value
                    } else {
                        let mut args = TokenStream::new();
                        args.push(Literal::string(&attr));
                        args.write(",");
                        args.extend(expr.stream);

                        let mut expr = TokenStream::new();

                        expr.write("::kobold::attribute::AttributeNode::new");
                        expr.push(Group::new(Delimiter::Parenthesis, args));

                        let value = gen.add_expression(expr);

                        let _ = writeln!(el.code, "{var}.setAttributeNode({value});");

                        value
                    };

                    el.args.push(value);
                }),
            };
        }

        if let Some(children) = self.children {
            let append = append(gen, &mut el.code, &mut el.args, children);
            let _ = writeln!(el.code, "{var}.{append};");
        }

        DomNode::Element(el)
    }
}
