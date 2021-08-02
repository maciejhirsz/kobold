use std::borrow::Cow;
use proc_macro::{TokenStream, TokenTree, Delimiter, Span};
use proc_macro::token_stream::IntoIter as TokenIter;
use quote::quote_spanned;
use crate::dom::{Node, Element, Field};
use crate::gen::IdentFactory;

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
        let msg = self.msg;
        let span = self.tt.as_ref().map(|tt| tt.span()).unwrap_or_else(|| Span::call_site()).into();
        (quote_spanned! { span =>
            fn _parse_error() {
                compile_error!(#msg)
            }
        }).into()
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
    types_factory: IdentFactory,
    names_factory: IdentFactory,
    pub fields: Vec<Field>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            types_factory: IdentFactory::new('A'),
            names_factory: IdentFactory::new('a'),
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
            },
            Err(err) if err.tt.is_none() => Ok(node),
            err => err,
        }
    }

    fn parse_node(&mut self, iter: &mut TokenIter) -> Result<Node, ParseError> {
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {
                Ok(Node::Element(self.parse_element(iter)?))
            },
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                let (_, typ) = self.types_factory.next();
                let (_, name) = self.names_factory.next();

                self.fields.push(Field {
                    typ,
                    name,
                    expr: group.stream().into(),
                });

                Ok(Node::Expression)
            },
            Some(TokenTree::Literal(lit)) => {
                let stringified = lit.to_string();

                let mut chars = stringified.chars();

                let text = match chars.next() {
                    // Take the string verbatim
                    Some('"') | Some('\'') => stringified,
                    _ => {
                        let mut quoted = String::with_capacity(stringified.len() + 2);

                        quoted.push('"');
                        quoted.push_str(&stringified);
                        quoted.push('"');

                        quoted
                    }
                };

                Ok(Node::Text(text))
            },
            tt => Err(ParseError::new("Expected an element, {expression}, or a string literal", tt)),
        }
    }

    fn parse_element(&mut self, iter: &mut TokenIter) -> Result<Element, ParseError> {
        let (tag, _) = expect_ident(iter.next())?;

        let mut element = Element {
            tag,
            props: Vec::new(),
            children: Vec::new(),
        };

        // Props loop
        loop {
            match iter.next() {
                Some(TokenTree::Ident(key)) => {
                    let key = key.to_string();

                    expect_punct(iter.next(), '=')?;

                    match iter.next() {
                        Some(value) => {
                            element.props.push((key, TokenStream::from(value).into()));
                        },
                        tt => return Err(tt.into()),
                    }
                },
                Some(TokenTree::Punct(punct)) if punct.as_char() == '/' => {
                    expect_punct(iter.next(), '>')?;

                    // Self-closing tag, no need to parse further
                    return Ok(element);
                },
                Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {
                    break;
                },
                tt => return Err(ParseError::new("Expected identifier, /, or >", tt))
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
                format!("Expected a closing tag for {}, but got {} instead", element.tag, closing),
                Some(tt),
            ));
        }

        expect_punct(iter.next(), '>')?;

        Ok(element)
    }
}

fn expect_punct(tt: Option<TokenTree>, expect: char) -> Result<(), ParseError> {
    match tt {
        Some(TokenTree::Punct(punct)) if punct.as_char() == expect => Ok(()),
        tt => Err(ParseError::new(format!("Expected {}", expect), tt)),
    }
}

fn expect_ident(tt: Option<TokenTree>) -> Result<(String, TokenTree), ParseError> {
    match tt {
        Some(TokenTree::Ident(ident)) => Ok((ident.to_string(), TokenTree::Ident(ident))),
        tt => Err(ParseError::new("Expected identifier", tt)),
    }
}
