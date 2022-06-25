use std::convert::TryFrom;

use arrayvec::ArrayString;
use beef::Cow;
use proc_macro::{Delimiter, Ident, Spacing, Span, TokenStream, TokenTree};
use proc_macro2::TokenStream as QuoteTokens;
use quote::{quote, quote_spanned};

use crate::dom::{Attribute, AttributeValue, Element, Field, FieldKind, Node};
use crate::gen::literal_to_string;
use crate::parse::*;
use crate::syntax::InlineCallback;

#[derive(Debug)]
pub struct ParseError {
    pub msg: Cow<'static, str>,
    pub tt: Option<TokenTree>,
}

impl ParseError {
    pub fn new<S: Into<Cow<'static, str>>>(msg: S, tt: Option<TokenTree>) -> Self {
        let mut error = ParseError::from(tt);

        error.msg = msg.into();
        error
    }

    pub fn tokenize(self) -> TokenStream {
        let msg = self.msg.as_ref();
        let span = self
            .tt
            .as_ref()
            .map(|tt| tt.span())
            .unwrap_or_else(Span::call_site)
            .into();

        (quote_spanned! { span =>
            fn _parse_error() {
                compile_error!(#msg)
            }
        })
        .into()
    }
}

impl From<Option<TokenTree>> for ParseError {
    fn from(tt: Option<TokenTree>) -> Self {
        ParseError {
            msg: "Unexpected token".into(),
            tt,
        }
    }
}

pub struct Parser {
    vars: usize,
    pub fields: Vec<Field>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            vars: 0,
            fields: Vec::new(),
        }
    }

    pub fn parse(&mut self, tokens: TokenStream) -> Result<Node, ParseError> {
        let mut iter = tokens.into_iter().peekable();

        let node = self.parse_node(&mut iter)?;

        // Convert to fragment if necessary
        match self.parse_node(&mut iter) {
            Ok(second) => {
                let mut fragment = vec![node, second];

                loop {
                    match self.parse_node(&mut iter) {
                        Ok(node) => fragment.push(node),
                        Err(err) if err.tt.is_none() => break,
                        err => return err,
                    }
                }

                Ok(Node::Fragment(fragment))
            }
            Err(err) if err.tt.is_none() => Ok(node),
            err => err,
        }
    }

    fn parse_node(&mut self, iter: &mut ParseStream) -> Result<Node, ParseError> {
        match iter.next() {
            Some(ref tt) if tt.is('<') => {
                let tag_ident: Ident = iter.parse()?;
                let tag = tag_ident.to_string();

                let mut el = self.parse_element(tag, iter)?;

                let render_call = match el.children_raw.take() {
                    Some(children) => quote! { render_with(#children) },
                    None => quote! { render() },
                };

                if el.is_component() {
                    let mut tag = into_quote(tag_ident);

                    if let Some(generics) = el.generics {
                        tag = quote! { #tag :: #generics };
                    }

                    let expr = match (el.attributes.is_empty(), el.defaults) {
                        (true, true) => quote! { #tag::default().#render_call },
                        (true, false) => quote! { #tag.#render_call },
                        (false, defaults) => {
                            let props = el
                                .attributes
                                .into_iter()
                                .map(|attr| {
                                    let name = into_quote(attr.ident);
                                    let value = match attr.value {
                                        AttributeValue::Literal(text) => quote! { #text },
                                        AttributeValue::Hoisted(_, expr) => expr,
                                        AttributeValue::Expression(expr) => expr,
                                    };

                                    quote! { #name: #value, }
                                })
                                .collect::<QuoteTokens>();

                            if defaults {
                                quote! {
                                    #tag {
                                        #props
                                        ..Default::default()
                                    }
                                    .#render_call
                                }
                            } else {
                                quote! {
                                    #tag {
                                        #props
                                    }
                                    .#render_call
                                }
                            }
                        }
                    };

                    self.new_field(FieldKind::Html, expr);

                    Ok(Node::Expression)
                } else {
                    for attr in el.attributes.iter() {
                        let attr_name = attr.name.as_str();

                        match &attr.value {
                            AttributeValue::Expression(tokens) => {
                                let (kind, expr) = match attr_name {
                                    "style" => (
                                        FieldKind::AttrNode,
                                        quote! { ::kobold::attribute::Style(#tokens) },
                                    ),
                                    "class" => (
                                        FieldKind::AttrNode,
                                        quote! { ::kobold::attribute::Class(#tokens) },
                                    ),
                                    n if n.starts_with("on") && n.len() > 2 => {
                                        let event_target = match el.tag.as_str() {
                                            "a" => "HtmlLinkElement",
                                            "form" => "HtmlFormElement",
                                            "img" => "HtmlImageElement",
                                            "input" => "HtmlInputElement",
                                            "option" => "HtmlOptionElement",
                                            "select" => "HtmlSelectElement",
                                            "textarea" => "HtmlTextAreaElement",
                                            _ => "HtmlElement",
                                        };

                                        let mut callback = TokenStream::from(tokens.clone())
                                            .into_iter()
                                            .peekable();

                                        // let inline_callback = InlineCallback::parse(&mut callback)?;

                                        // panic!("{} {}", inline_callback.invocation, inline_callback.arg);

                                        let expr = if let Ok(inline_callback) =
                                            InlineCallback::parse(&mut callback)
                                        {
                                            let mut expr = inline_callback.invocation;

                                            expr.write(&format!("::<::kobold::reexport::web_sys::{event_target}, _, _>"));
                                            expr.push_tt(inline_callback.arg);

                                            QuoteTokens::from(expr)
                                        } else {
                                            let event_target =
                                                quote::format_ident!("{event_target}");

                                            quote! {{
                                                let constrained: ::kobold::stateful::Callback<
                                                    ::kobold::reexport::web_sys::#event_target,
                                                    _,
                                                    _,
                                                > = #tokens;

                                                constrained
                                            }}
                                        };

                                        (FieldKind::Callback(n[2..].into()), expr)
                                    }
                                    _ => (
                                        FieldKind::AttrNode,
                                        quote! {
                                            ::kobold::attribute::AttributeNode::new(#attr_name, #tokens)
                                        },
                                    ),
                                };

                                self.new_field(kind, expr);
                            }
                            AttributeValue::Hoisted(typ, tokens) => {
                                let field = self
                                    .new_field(FieldKind::AttrHoisted(typ.clone()), tokens.clone());

                                el.hoisted_attrs.push(field.name.clone());
                            }
                            _ => (),
                        }
                    }

                    Ok(Node::Element(el))
                }
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let expr = into_quote(group);
                self.new_field(FieldKind::Html, expr);

                Ok(Node::Expression)
            }
            Some(TokenTree::Literal(lit)) => Ok(Node::Text(literal_to_string(lit))),
            tt => Err(ParseError::new(
                "Expected an element, a literal value, or an {expression}",
                tt,
            )),
        }
    }

    fn parse_element(
        &mut self,
        tag: String,
        iter: &mut ParseStream,
    ) -> Result<Element, ParseError> {
        let mut element = Element::new(tag);

        if iter.allow('<') {
            let generics: Generics = iter.parse()?;

            element.generics = Some(generics.tokens.into());
        }

        // Props loop
        loop {
            match iter.next() {
                Some(TokenTree::Ident(ident)) => {
                    let name = ident.to_string();

                    iter.expect('=')?;

                    let value = match iter.next() {
                        Some(TokenTree::Literal(lit)) => AttributeValue::Literal(into_quote(lit)),
                        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                            AttributeValue::from_group(name.as_str(), group.stream().into())
                        }
                        Some(tt) => {
                            return Err(ParseError::new(
                                "Expected a literal value, or an {expession}",
                                Some(tt),
                            ));
                        }
                        None => {
                            return Err(ParseError::new(
                                "Missing attribute value",
                                Some(TokenTree::Ident(ident)),
                            ))
                        }
                    };

                    element.attributes.push(Attribute { name, ident, value });
                }
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    let mut iter = group.stream().into_iter().peekable();

                    let ident: Ident = iter.parse()?;
                    let name = ident.to_string();
                    let value =
                        AttributeValue::from_group(name.as_str(), into_quote(ident.clone()));

                    expect_end(
                        iter.next(),
                        "Shorthand attributes can only contain a single variable name",
                    )?;

                    element.attributes.push(Attribute { name, ident, value });
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                    if punct.spacing() == Spacing::Joint {
                        iter.expect('.')?;
                        iter.expect('/')?;
                        iter.expect('>')?;

                        element.defaults = true;

                        return Ok(element);
                    }

                    let name = "class".to_string();
                    let ident = Ident::new(&name, punct.span());
                    let value = AttributeValue::css_attribute_value(&name, iter)?;

                    element.attributes.push(Attribute { name, ident, value })
                }
                Some(tt) if tt.is('#') => {
                    let name = "id".to_string();
                    let ident = Ident::new(&name, tt.span());
                    let value = AttributeValue::css_attribute_value(&name, iter)?;

                    element.attributes.push(Attribute { name, ident, value })
                }
                tt if tt.is('/') => {
                    iter.expect('>')?;

                    // Self-closing tag, no need to parse further
                    return Ok(element);
                }
                tt if tt.is('>') => break,
                tt => return Err(ParseError::new("Expected identifier, /, or >", tt)),
            }
        }

        let mut children = TokenStream::new();
        let mut depth = 0;

        while let Some(tt) = iter.next() {
            if tt.is('<') {
                if let Some(next) = iter.next() {
                    if next.is('/') {
                        if depth == 0 {
                            break;
                        }

                        depth -= 1;

                        let ident = iter.parse()?;

                        children.extend([tt, next, TokenTree::Ident(ident)]);
                    } else {
                        depth += 1;

                        children.extend([tt, next]);
                    }
                }

                // Allow generics after ident
                if iter.allow('<') {
                    let generics: Generics = iter.parse()?;

                    children.extend(generics.tokens);
                }

                while let Some(tt) = iter.next() {
                    if tt.is('/') {
                        depth -= 1;
                    } else if tt.is('>') {
                        children.extend([tt]);
                        break;
                    }

                    children.extend([tt]);
                }

                continue;
            }

            children.extend([tt]);
        }

        if element.is_component() {
            let parsed = crate::html(children);

            element.children_raw = Some(parsed.into());
        } else {
            // panic!("{children}");
            let mut iter = children.into_iter().peekable();

            loop {
                match self.parse_node(&mut iter) {
                    Ok(child) => element.children.push(child),
                    Err(err) => match err.tt {
                        None => break,
                        _ => return Err(err),
                    },
                }
            }
        }

        iter.expect(element.tag.as_str())?;
        iter.expect('>')?;

        Ok(element)
    }

    fn new_field(&mut self, kind: FieldKind, expr: QuoteTokens) -> &Field {
        const LETTERS: usize = 26;

        // This gives us up to 456976 unique identifiers, should be enough :)
        let mut buf = ArrayString::<4>::new();
        let mut n = self.vars;

        self.vars += 1;

        loop {
            buf.push((u8::try_from(n % LETTERS).unwrap() + b'A') as char);

            n /= LETTERS;

            if n == 0 {
                break;
            }
        }

        let typ = into_quote(Ident::new(&buf, Span::call_site()));

        buf.make_ascii_lowercase();

        let name = into_quote(Ident::new(&buf, Span::call_site()));

        self.fields.push(Field {
            kind,
            typ,
            name,
            expr,
        });

        self.fields.last().unwrap()
    }
}

pub fn into_quote(tt: impl Into<TokenTree>) -> QuoteTokens {
    TokenStream::from(tt.into()).into()
}

fn expect_end(tt: Option<TokenTree>, err: &'static str) -> Result<(), ParseError> {
    match tt {
        None => Ok(()),
        tt => Err(ParseError::new(err, tt)),
    }
}
