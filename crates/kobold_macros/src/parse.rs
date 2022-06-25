use beef::Cow;
use proc_macro::{Delimiter, Ident, Literal, Spacing, Span, TokenStream, TokenTree};

use crate::parser::ParseError;

pub type ParseStream = std::iter::Peekable<proc_macro::token_stream::IntoIter>;

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

    fn parse<T: Parse>(&mut self) -> Result<T, ParseError>;

    fn end(&mut self) -> bool;
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

    fn parse<T: Parse>(&mut self) -> Result<T, ParseError> {
        T::parse(self)
    }

    fn end(&mut self) -> bool {
        self.peek().is_none()
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

pub struct Generics {
    pub tokens: TokenStream,
}

impl Parse for Generics {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let opening = stream.expect('>')?;

        let mut depth = 0;
        let mut tokens = TokenStream::new();

        tokens.extend([opening]);

        for token in stream {
            if token.is('<') {
                depth += 1;
            } else if token.is('>') {
                depth -= 1;

                if depth == 0 {
                    tokens.extend([token]);

                    return Ok(Generics { tokens });
                }
            }

            tokens.extend([token]);
        }

        Err(ParseError::new(
            "Missing closing > for generics",
            tokens.into_iter().next(),
        ))
    }
}

pub struct CssLabel {
    pub label: String,
    pub span: Span,
}

impl Parse for CssLabel {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        use std::fmt::Write;

        let mut ident: Ident = stream.parse()?;

        let mut label = String::new();
        let span = ident.span();

        write!(&mut label, "{ident}").unwrap();

        while stream.allow('-') {
            stream.next();

            ident = stream.parse()?;

            write!(&mut label, "-{ident}").unwrap();
        }

        Ok(CssLabel { label, span })
    }
}

impl CssLabel {
    pub fn into_literal(self) -> Literal {
        let mut lit = Literal::string(&self.label);
        lit.set_span(self.span);
        lit
    }
}
