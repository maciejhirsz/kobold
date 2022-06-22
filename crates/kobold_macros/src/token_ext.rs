use beef::Cow;
use proc_macro::{Ident, TokenTree};

use crate::parser::ParseError;

pub trait Pattern: Copy {
    fn matches(self, tt: &TokenTree) -> bool;

    fn expected(self) -> Cow<'static, str>;
}

pub trait Parse: Sized {
    fn parse(tt: Option<TokenTree>) -> Result<Self, Option<TokenTree>>;

    fn expected() -> Cow<'static, str>;
}

impl Parse for Ident {
    fn parse(tt: Option<TokenTree>) -> Result<Self, Option<TokenTree>> {
        match tt {
            Some(TokenTree::Ident(ident)) => Ok(ident),
            tt => Err(tt),
        }
    }

    fn expected() -> Cow<'static, str> {
        "Expected an identifier".into()
    }
}

impl Pattern for &str {
    fn matches(self, tt: &TokenTree) -> bool {
        match tt {
            TokenTree::Ident(ident) => ident.to_string() == self,
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

pub trait IteratorExt {
    fn expect(&mut self, pattern: impl Pattern) -> Result<TokenTree, ParseError>;

    fn parse<T: Parse>(&mut self) -> Result<T, ParseError>;
}

impl<I> IteratorExt for I
where
    I: Iterator<Item = TokenTree>,
{
    fn expect(&mut self, pattern: impl Pattern) -> Result<TokenTree, ParseError> {
        match self.next() {
            Some(tt) if pattern.matches(&tt) => Ok(tt),
            tt => Err(ParseError::new(pattern.expected(), tt))
        }
    }

    fn parse<T: Parse>(&mut self) -> Result<T, ParseError> {
        T::parse(self.next()).map_err(|tt| ParseError::new(T::expected(), tt))
    }
}
