use std::convert::TryFrom;

use arrayvec::ArrayString;
use beef::Cow;
use proc_macro::token_stream::IntoIter as TokenIter;
use proc_macro::{Delimiter, Ident, Literal, Spacing, Span, TokenStream, TokenTree};
use proc_macro2::TokenStream as QuoteTokens;
use quote::{quote, quote_spanned};

use crate::dom::{Attribute, AttributeValue, Element, Field, FieldKind, Node};

#[derive(Debug)]
pub struct ParseError {
    msg: Cow<'static, str>,
    tt: Option<TokenTree>,
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
        let mut iter = tokens.into_iter();

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

    fn parse_node(&mut self, iter: &mut TokenIter) -> Result<Node, ParseError> {
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {
                let (tag, tag_ident) = expect_ident(iter.next())?;

                let mut el = self.parse_element(tag, iter)?;

                let render_call = match el.children_raw.take() {
                    Some(children) => quote! { render_with(#children) },
                    None => quote! { render() },
                };

                if el.is_component() {
                    let mut tag = into_quote(tag_ident);

                    if let Some(generics) = el.generics {
                        tag = quote! { #tag :: <#generics> };
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
                                        AttributeValue::Text(text) => quote! { #text },
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
                        if let AttributeValue::Expression(tokens) = &attr.value {
                            let attr_name = attr.name.as_str();

                            let (kind, expr) = match attr_name {
                                "style" => (
                                    FieldKind::Attr,
                                    quote! { ::kobold::attribute::Style(#tokens) },
                                ),
                                "class" => (
                                    FieldKind::Attr,
                                    quote! { ::kobold::attribute::Class(#tokens) },
                                ),
                                n if n.starts_with("on") && n.len() > 2 => {
                                    (FieldKind::Callback(n[2..].into()), tokens.clone())
                                }
                                _ => (
                                    FieldKind::Attr,
                                    quote! {
                                        ::kobold::attribute::Attribute::new(#attr_name, #tokens)
                                    },
                                ),
                            };

                            self.new_field(kind, expr);
                        }
                    }

                    Ok(Node::Element(el))
                }
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let expr = into_quote(group);
                self.new_field(FieldKind::Html, quote! { #expr.into_html() });

                Ok(Node::Expression)
            }
            Some(TokenTree::Literal(lit)) => Ok(Node::Text(literal_to_string(lit))),
            tt => Err(ParseError::new(
                "Expected an element, a literal value, or an {expression}",
                tt,
            )),
        }
    }

    fn parse_element(&mut self, tag: String, iter: &mut TokenIter) -> Result<Element, ParseError> {
        let mut element = Element {
            tag,
            generics: None,
            attributes: Vec::new(),
            children: Vec::new(),
            children_raw: None,
            defaults: false,
        };

        let mut next = iter.next();

        match next {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {
                element.generics = Some(
                    iter.take_while(
                        |p| !matches!(p, TokenTree::Punct(punct) if punct.as_char() == '>'),
                    )
                    .collect::<TokenStream>()
                    .into(),
                );

                next = iter.next();
            }
            _ => (),
        }

        // Props loop
        loop {
            match next {
                Some(TokenTree::Ident(ident)) => {
                    let name = ident.to_string();

                    expect_punct(iter.next(), '=')?;

                    let value = match iter.next() {
                        Some(TokenTree::Literal(lit)) => {
                            AttributeValue::Text(literal_to_string(lit))
                        }
                        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                            AttributeValue::Expression(group.stream().into())
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
                Some(TokenTree::Group(group)) => {
                    let mut iter = group.stream().into_iter();

                    let (name, ident) = expect_ident(iter.next())?;
                    expect_end(
                        iter.next(),
                        "Shorthand attributes can only contain a single variable name",
                    )?;

                    element.attributes.push(Attribute {
                        name,
                        ident: ident.clone(),
                        value: AttributeValue::Expression(into_quote(ident)),
                    });
                }
                Some(TokenTree::Punct(punct))
                    if punct.as_char() == '.' && punct.spacing() == Spacing::Joint =>
                {
                    expect_punct(iter.next(), '.')?;
                    expect_punct(iter.next(), '/')?;
                    expect_punct(iter.next(), '>')?;

                    element.defaults = true;

                    return Ok(element);
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '/' => {
                    expect_punct(iter.next(), '>')?;

                    // Self-closing tag, no need to parse further
                    return Ok(element);
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {
                    break;
                }
                tt => return Err(ParseError::new("Expected identifier, /, or >", tt)),
            }

            next = iter.next();
        }

        let mut children = TokenStream::new();
        let mut stack = 0;

        while let Some(tt) = iter.next() {
            if punct(&tt) == Some('<') {
                if let Some(next) = iter.next() {
                    if punct(&next) == Some('/') {
                        if stack == 0 {
                            break;
                        }

                        stack -= 1;
                    } else {
                        stack += 1;
                    }

                    children.extend([tt, next]);
                }

                let next = iter.next();
                let mut p = next.as_ref().and_then(punct);

                children.extend(next);

                // Allow generics after ident
                if p == Some('<') {
                    while let Some(next) = iter.next() {
                        let punct = punct(&next);

                        children.extend([next]);

                        // TODO: handle nested generics
                        if punct == Some('>') {
                            break;
                        }
                    }
                }

                loop {
                    match p {
                        Some('/') => stack -= 1,
                        Some('>') => break,
                        _ => ()
                    }

                    match iter.next() {
                        Some(tt) => {
                            p = punct(&tt);
                            children.extend([tt]);
                        },
                        None => break,
                    }
                }

                while let Some(next) = iter.next() {
                    let punct = punct(&next);

                    children.extend([next]);

                    match punct {
                        Some('/') => stack -= 1,
                        Some('>') => break,
                        _ => ()
                    }
                }

                continue;
            }

            children.extend([tt]);
        }


        if element.is_component() {
            let parsed = crate::html(children);

            element.children_raw = Some(parsed.into());
        } else {
            let mut iter = children.into_iter();

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

        let (closing, tt) = expect_ident(iter.next())?;

        if closing != element.tag {
            return Err(ParseError::new(
                format!(
                    "Expected a closing tag for {}, but got {} instead",
                    element.tag, closing
                ),
                Some(TokenTree::Ident(tt)),
            ));
        }

        expect_punct(iter.next(), '>')?;

        Ok(element)
    }

    fn new_field(&mut self, kind: FieldKind, expr: QuoteTokens) {
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
    }
}

fn literal_to_string(lit: Literal) -> String {
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

fn into_quote(tt: impl Into<TokenTree>) -> QuoteTokens {
    TokenStream::from(tt.into()).into()
}

fn expect_end(tt: Option<TokenTree>, err: &'static str) -> Result<(), ParseError> {
    match tt {
        None => Ok(()),
        tt => Err(ParseError::new(err, tt)),
    }
}

fn punct(tt: &TokenTree) -> Option<char> {
    match tt {
        TokenTree::Punct(p) => Some(p.as_char()),
        _ => None,
    }
}

fn expect_punct(tt: Option<TokenTree>, expect: char) -> Result<(), ParseError> {
    match tt {
        Some(TokenTree::Punct(punct)) if punct.as_char() == expect => Ok(()),
        tt => Err(ParseError::new(format!("Expected {}", expect), tt)),
    }
}

fn expect_ident(tt: Option<TokenTree>) -> Result<(String, Ident), ParseError> {
    match tt {
        Some(TokenTree::Ident(ident)) => Ok((ident.to_string(), ident)),
        tt => Err(ParseError::new("Expected identifier", tt)),
    }
}
