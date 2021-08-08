use crate::dom::{Attribute, AttributeValue, Element, Field, FieldKind, Node};

use arrayvec::ArrayString;
use beef::Cow;
use proc_macro::token_stream::IntoIter as TokenIter;
use proc_macro::{Delimiter, Ident, Literal, Span, TokenStream, TokenTree};
use proc_macro2::TokenStream as QuoteTokens;
use quote::{quote, quote_spanned};
use std::convert::TryFrom;

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

                let el = self.parse_element(tag, iter)?;

                if el.is_component() {
                    let tag = into_quote(tag_ident);

                    let props = if el.attributes.is_empty() {
                        quote! { Properties }
                    } else {
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

                        quote! {
                            Properties {
                                #props
                            }
                        }
                    };

                    let expr = quote! {
                        ::kobold::internals::WrappedProperties::<#tag, _, _>::new(
                            {
                                type Properties = <#tag as ::kobold::Component>::Properties;

                                #props
                            },
                            #tag::render,
                        )
                    };

                    self.new_field(FieldKind::Html, expr);

                    Ok(Node::Expression)
                } else {
                    for attr in el.attributes.iter() {
                        if let AttributeValue::Expression(tokens) = &attr.value {
                            let attr_name = attr.name.as_str();

                            let (kind, constructor) = match attr_name {
                                "style" => (FieldKind::Attr, Some(quote! { Style })),
                                "class" => (FieldKind::Attr, Some(quote! { Class })),
                                n if n.starts_with("on") && n.len() > 2 => (
                                    FieldKind::Callback(n[2..].into()),
                                    Some(quote! { Callback }),
                                ),
                                _ => (FieldKind::Attr, None),
                            };

                            let expr = match constructor {
                                Some(constructor) => quote! {
                                    ::kobold::attribute::#constructor(#tokens)
                                },
                                None => quote! {
                                    ::kobold::attribute::Attribute::new(#attr_name, #tokens)
                                },
                            };

                            self.new_field(kind, expr);
                        }
                    }

                    Ok(Node::Element(el))
                }
            }
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let mut iter = group.stream().into_iter();

                let expr = match iter.next() {
                    Some(TokenTree::Ident(ref ident)) if ident.to_string() == "for" => {
                        let tokens: QuoteTokens = iter.collect::<TokenStream>().into();

                        quote! {
                            ::kobold::IterWrapper(#tokens)
                        }
                    }
                    Some(tt) => IntoIterator::into_iter([tt])
                        .chain(iter)
                        .collect::<TokenStream>()
                        .into(),
                    None => quote! {},
                };

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

    fn parse_element(&mut self, tag: String, iter: &mut TokenIter) -> Result<Element, ParseError> {
        let mut element = Element {
            tag,
            attributes: Vec::new(),
            children: Vec::new(),
        };

        // Props loop
        loop {
            match iter.next() {
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
        }

        // Children loop
        loop {
            match self.parse_node(iter) {
                Ok(child) => element.children.push(child),
                Err(err) => match err.tt {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '/' => break,
                    _ => return Err(err),
                },
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
