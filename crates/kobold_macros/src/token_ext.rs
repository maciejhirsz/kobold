use beef::Cow;
use proc_macro::{Ident, TokenTree};

use crate::parser::ParseError;

pub trait Pattern: Copy {
    fn matches(self, tt: &Option<TokenTree>) -> bool;

    fn expected(self) -> Cow<'static, str>;
}

pub trait TyPattern: Sized {
    fn matches(tt: Option<TokenTree>) -> Result<Self, Option<TokenTree>>;

    fn expected() -> Cow<'static, str>;
}

impl TyPattern for Ident {
    fn matches(tt: Option<TokenTree>) -> Result<Self, Option<TokenTree>> {
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
    fn matches(self, tt: &Option<TokenTree>) -> bool {
        match tt {
            Some(TokenTree::Ident(ident)) => ident.to_string() == self,
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        format!("Expected {self}").into()
    }
}

impl Pattern for char {
    fn matches(self, tt: &Option<TokenTree>) -> bool {
        match tt {
            Some(TokenTree::Punct(punct)) => punct.as_char() == self,
            _ => false,
        }
    }

    fn expected(self) -> Cow<'static, str> {
        format!("Expected {self}").into()
    }
}

pub trait IteratorExt {
    fn expect(&mut self, pattern: impl Pattern) -> Result<(), ParseError>;

    fn expect_ty<T: TyPattern>(&mut self) -> Result<T, ParseError>;
}

impl<I> IteratorExt for I
where
    I: Iterator<Item = TokenTree>,
{
    // fn cloning(self, into: &mut TokenStream) -> Cloning<I> {
    //     Cloning {
    //         iter: self,
    //         into,
    //     }
    // }

    fn expect(&mut self, pattern: impl Pattern) -> Result<(), ParseError> {
        let tt = self.next();

        if pattern.matches(&tt) {
            Ok(())
        } else {
            Err(ParseError {
                msg: pattern.expected(),
                tt,
            })
        }
    }

    fn expect_ty<T: TyPattern>(&mut self) -> Result<T, ParseError> {
        T::matches(self.next()).map_err(|tt| ParseError {
            msg: T::expected(),
            tt,
        })
    }
}

// struct Cloning {
//     iter: self
// }
