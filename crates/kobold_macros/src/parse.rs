//! [`ParseStream`](ParseStream), the [`Parse`](Parse) trait and utilities for working with
//! token streams without `syn` or `quote`.

use std::cell::RefCell;
use std::fmt::{Display, Write};

use beef::Cow;
use proc_macro::{Delimiter, Ident, Spacing, Span, TokenStream, TokenTree};

use crate::dom2::{ShallowNodeIter, ShallowStream};

pub type ParseStream = std::iter::Peekable<proc_macro::token_stream::IntoIter>;

pub mod prelude {
    pub use super::{DisplayExt, IteratorExt, TokenStreamExt, TokenTreeExt};
    pub use super::{Lit, Parse, ParseError, ParseStream};
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: Cow<'static, str>,
    pub span: Span,
}

pub trait IntoSpan {
    fn into_span(self) -> Span;
}

impl IntoSpan for Span {
    fn into_span(self) -> Span {
        self
    }
}

impl IntoSpan for TokenTree {
    fn into_span(self) -> Span {
        self.span()
    }
}

impl IntoSpan for Option<TokenTree> {
    fn into_span(self) -> Span {
        self.as_ref().map(TokenTree::span).unwrap_or_else(Span::call_site)
    }
}

impl ParseError {
    pub fn new<M, S>(msg: M, spannable: S) -> Self
    where
        M: Into<Cow<'static, str>>,
        S: IntoSpan,
    {
        ParseError {
            msg: msg.into(),
            span: spannable.into_span(),
        }
    }

    pub fn tokenize(self) -> TokenStream {
        let msg = self.msg.as_ref();
        let span = self.span.into();

        (quote::quote_spanned! { span =>
            fn _parse_error() {
                compile_error!(#msg)
            }
        })
        .into()
    }
}

pub trait Pattern: Copy {
    fn matches(self, tt: &TokenTree) -> bool;

    fn expected(self) -> Cow<'static, str>;
}

pub trait Parse: Sized {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError>;
}

impl Parse for Ident {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        match stream.next() {
            Some(TokenTree::Ident(ident)) => Ok(ident),
            tt => Err(ParseError::new("Expected an identifier", tt)),
        }
    }
}

impl Parse for () {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        match stream.next() {
            Some(tt) => Err(ParseError::new("Unexpected token", tt)),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Lit;

impl Pattern for Lit {
    fn matches(self, tt: &TokenTree) -> bool {
        match tt {
            TokenTree::Literal(_) => true,
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        "Expected a literal value".into()
    }
}

impl Pattern for &str {
    fn matches(self, tt: &TokenTree) -> bool {
        match tt {
            TokenTree::Ident(ident) => ident.str_eq(self),
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        format!("Expected {self}").into()
    }
}

impl Pattern for char {
    fn matches(self, tt: &TokenTree) -> bool {
        match tt {
            TokenTree::Punct(punct) => punct.as_char() == self,
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        format!("Expected {self}").into()
    }
}

impl Pattern for (char, Spacing) {
    fn matches(self, tt: &TokenTree) -> bool {
        match tt {
            TokenTree::Punct(punct) => punct.as_char() == self.0 && punct.spacing() == self.1,
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        format!("Expected {}", self.0).into()
    }
}

impl Pattern for Delimiter {
    fn matches(self, tt: &TokenTree) -> bool {
        match tt {
            TokenTree::Group(group) => group.delimiter() == self,
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        match self {
            Delimiter::Parenthesis => "Expected (...)",
            Delimiter::Brace => "Expected {...}",
            Delimiter::Bracket => "Expected [...]",
            Delimiter::None => "Expected a group",
        }
        .into()
    }
}

pub trait IteratorExt {
    fn expect(&mut self, pattern: impl Pattern) -> Result<TokenTree, ParseError>;

    fn allow(&mut self, pattern: impl Pattern) -> bool;

    fn allow_consume(&mut self, pattern: impl Pattern) -> Option<TokenTree>;

    fn parse<T: Parse>(&mut self) -> Result<T, ParseError>;

    fn end(&mut self) -> bool;

    fn into_shallow_stream(self) -> ShallowStream;
}

impl IteratorExt for ParseStream {
    fn expect(&mut self, pattern: impl Pattern) -> Result<TokenTree, ParseError> {
        match self.next() {
            Some(tt) if pattern.matches(&tt) => Ok(tt),
            tt => Err(ParseError::new(pattern.expected(), tt)),
        }
    }

    fn allow(&mut self, pattern: impl Pattern) -> bool {
        self.peek().map(|tt| pattern.matches(tt)).unwrap_or(false)
    }

    fn allow_consume(&mut self, pattern: impl Pattern) -> Option<TokenTree> {
        self.next_if(|tt| pattern.matches(tt))
    }

    fn parse<T: Parse>(&mut self) -> Result<T, ParseError> {
        T::parse(self)
    }

    fn end(&mut self) -> bool {
        self.peek().is_none()
    }

    fn into_shallow_stream(self) -> ShallowStream {
        ShallowNodeIter::new(self).peekable()
    }
}

pub trait TokenTreeExt {
    fn is(&self, pattern: impl Pattern) -> bool;
}

impl TokenTreeExt for TokenTree {
    fn is(&self, pattern: impl Pattern) -> bool {
        pattern.matches(self)
    }
}

impl TokenTreeExt for Option<TokenTree> {
    fn is(&self, pattern: impl Pattern) -> bool {
        self.as_ref().map(|tt| pattern.matches(tt)).unwrap_or(false)
    }
}

pub trait TokenStreamExt {
    fn write(&mut self, rust: &str);

    fn push(&mut self, tt: impl Into<TokenTree>);

    fn parse_stream(self) -> ParseStream;
}

impl TokenStreamExt for TokenStream {
    fn write(&mut self, rust: &str) {
        use std::str::FromStr;

        self.extend(TokenStream::from_str(rust).unwrap());
    }

    fn push(&mut self, tt: impl Into<TokenTree>) {
        self.extend([tt.into()]);
    }

    fn parse_stream(self) -> ParseStream {
        self.into_iter().peekable()
    }
}

thread_local! {
    static DISPLAY_EXT_BUF: RefCell<String> = RefCell::new(String::with_capacity(128));
}

pub trait DisplayExt: Display {
    /// Do stuff with `Display` types such as `Ident`s without
    /// allocating a new buffer with `to_string` every time.
    fn with_str<R, F: FnOnce(&str) -> R>(&self, f: F) -> R {
        DISPLAY_EXT_BUF.with(move |buf| {
            let mut buf = buf.borrow_mut();

            buf.clear();

            write!(&mut buf, "{self}").unwrap();

            f(&buf)
        })
    }

    fn str_eq(&self, other: &str) -> bool {
        self.with_str(|s| s == other)
    }
}

impl<T: Display> DisplayExt for T {}
