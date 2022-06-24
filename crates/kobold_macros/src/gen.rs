use std::fmt::{self, Write};
use std::marker::PhantomData;

use arrayvec::ArrayString;
use proc_macro::{Ident, Span, TokenStream, TokenTree};
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;

use crate::dom::{AttributeValue, Field, FieldKind, Node};

#[derive(Debug)]
pub enum Infallible {}

impl From<fmt::Error> for Infallible {
    fn from(err: fmt::Error) -> Infallible {
        panic!("{}", err)
    }
}

pub struct Generator<'a, I> {
    fields: I,
    var_count: usize,
    pub render: String,
    pub update: String,
    args: String,
    args_tokens: Vec<QuoteTokens>,
    _marker: PhantomData<&'a Field>,
}

impl<'a, I> Generator<'a, I>
where
    I: Iterator<Item = &'a Field> + 'a,
{
    pub fn new(fields: I) -> Self {
        Generator {
            fields,
            var_count: 0,
            render: String::new(),
            update: String::new(),
            args: String::new(),
            args_tokens: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn generate(&mut self, dom: &Node) -> Result<String, Infallible> {
        match dom {
            Node::Text(text) => {
                let e = self.next_el();

                write!(
                    &mut self.render,
                    "const {}=document.createTextNode({});",
                    e, text
                )?;

                Ok(e)
            }
            Node::Element(el) => {
                let e = self.next_el();

                macro_rules! js {
                    ($($t:tt)*) => {
                        write!(&mut self.render, $($t)*)?
                    };
                }

                js!("const {}=document.createElement({:?});", e, el.tag);

                self.append(&e, &el.children)?;

                for attr in el.attributes.iter() {
                    let value = match &attr.value {
                        AttributeValue::Literal(value) => literal_to_string(value),
                        AttributeValue::Hoisted(_, _) => {
                            let (arg, _) = self.next_arg();

                            arg
                        }
                        AttributeValue::Expression(_) => {
                            let (arg, field) = self.next_arg();

                            match &field.kind {
                                FieldKind::AttrNode => {
                                    js!("{}.setAttributeNode({});", e, arg);
                                }
                                FieldKind::Callback(event) => {
                                    let action = event.clone();
                                    js!("{}.addEventListener({:?},{});", e, action, arg);
                                }
                                FieldKind::Html => {
                                    panic!("HTML node in element attributes!");
                                }
                                FieldKind::AttrHoisted(_) => {
                                    panic!("Hoisted attribute with expression value!");
                                }
                            }

                            continue;
                        }
                    };

                    match attr.name.as_ref() {
                        "class" => {
                            js!("{}.className={};", e, value);
                        }
                        "style" | "id" | "checked" => {
                            js!("{}.{}={};", e, attr.name, value);
                        }
                        _ => {
                            js!("{}.setAttribute({:?},{});", e, attr.name, value)
                        }
                    }
                }

                Ok(e)
            }
            Node::Fragment(children) => {
                let e = self.next_el();

                write!(
                    &mut self.render,
                    "const {e}=document.createDocumentFragment();\
                    {e}.append({e}.$begin=document.createTextNode(\"\"));",
                )?;

                self.append(&e, children)?;

                write!(
                    &mut self.render,
                    "{e}.append({e}.$end=document.createTextNode(\"\"));",
                )?;

                Ok(e)
            }
            Node::Expression => Ok(self.next_arg().0),
        }
    }

    pub fn append(&mut self, el: &str, children: &[Node]) -> Result<(), Infallible> {
        let mut append = String::new();

        if let Some((first, rest)) = children.split_first() {
            match first {
                Node::Text(text) => append.push_str(text),
                ref node => append.push_str(&self.generate(node)?),
            }

            for child in rest {
                append.push(',');

                match child {
                    Node::Text(text) => append.push_str(text),
                    ref node => append.push_str(&self.generate(node)?),
                }
            }
        }

        write!(&mut self.render, "{}.append({});", el, append)?;

        Ok(())
    }

    pub fn render_js(&mut self, root: &str) -> (QuoteTokens, QuoteTokens) {
        const FN_PREFIX: &str = "__transient_";
        const FN_BUF_LEN: usize = FN_PREFIX.len() + 16;

        use std::hash::Hasher;
        let mut hasher = fnv::FnvHasher::default();

        hasher.write(self.render.as_bytes());

        let hash = hasher.finish();
        let mut fn_name = ArrayString::<FN_BUF_LEN>::new();

        fn_name.push_str(FN_PREFIX);

        write!(&mut fn_name, "{:x}", hash)
            .expect("transient function name buffer is too small, this is a bug, please report it");

        let js = format!(
            "export function {fn_name}({}){{{}return {root};}}",
            self.args, self.render
        );
        let args = &self.args_tokens;

        let fn_name = Ident::new(&fn_name, Span::call_site());
        let fn_name: QuoteTokens = TokenStream::from(TokenTree::Ident(fn_name)).into();

        (
            fn_name.clone(),
            quote! {
                #[wasm_bindgen::prelude::wasm_bindgen(inline_js = #js)]
                extern "C" {
                    fn #fn_name(#(#args),*) -> ::kobold::reexport::web_sys::Node;
                }
            },
        )
    }

    fn next_el(&mut self) -> String {
        let e = format!("e{}", self.var_count);
        self.var_count += 1;
        e
    }

    fn next_arg(&mut self) -> (String, &Field) {
        let field = self
            .fields
            .next()
            .expect("Trying to generate more arguments in JS than fields in Rust");

        let name = &field.name;
        let arg = name.to_string();

        if !self.args.is_empty() {
            self.args.push(',');
        }
        self.args.push_str(&arg);

        let arg_tokens = match &field.kind {
            FieldKind::AttrHoisted(typ) => {
                quote! { #name: #typ }
            }
            _ => quote! { #name: &wasm_bindgen::JsValue },
        };

        self.args_tokens.push(arg_tokens);

        (arg, field)
    }
}

pub fn literal_to_string(lit: impl ToString) -> String {
    const QUOTE: &str = "\"";

    let stringified = lit.to_string();

    match stringified.chars().next() {
        // Take the string verbatim
        Some('"' | '\'') => stringified,
        _ => {
            let mut buf = String::with_capacity(stringified.len() + QUOTE.len() * 2);

            buf.extend([QUOTE, &stringified, QUOTE]);
            buf
        }
    }
}
