//! `Parse` logic for different syntax elements

use std::fmt::Write;

use proc_macro::{Ident, Literal, Span, TokenStream, TokenTree};

use crate::parse::*;

/// Regular Rust `<Generic, Types>`, we don't care about what they are,
/// but we do care about nested angle braces.
pub struct Generics {
    pub tokens: TokenStream,
}

impl Parse for Generics {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let opening = stream.expect('<')?;

        let mut depth = 1;
        let mut tokens = TokenStream::new();

        tokens.push(opening);

        for token in stream {
            if token.is('<') {
                depth += 1;
            } else if token.is('>') {
                depth -= 1;

                if depth == 0 {
                    tokens.push(token);

                    return Ok(Generics { tokens });
                }
            }

            tokens.push(token);
        }

        Err(ParseError::new(
            "Missing closing > for generic type declaration",
            tokens.into_iter().next(),
        ))
    }
}

/// CSS-style label, matches sequences of identifiers with dashes allowed.
pub struct CssLabel {
    pub label: String,
    pub span: Span,
}

impl Parse for CssLabel {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut ident: Ident = stream.parse()?;
        let mut label = String::new();

        let span = ident.span();

        write!(&mut label, "{ident}").unwrap();

        while stream.allow_consume('-').is_some() {
            ident = stream.parse()?;

            write!(&mut label, "-{ident}").unwrap();
        }

        Ok(CssLabel { label, span })
    }
}

impl CssLabel {
    pub fn into_literal(self) -> Literal {
        let mut lit = Literal::string(&self.label);

        // Keep resolution to literal, but change location
        lit.set_span(lit.span().located_at(self.span));
        lit
    }
}

/// Matches `[<ident>.]*<ident>.bind(...)`. and splits it into the
/// invocation `ctx.bind` and the group token tree for closure `(...)`.
///
/// This is useful for adding type information to events:
///
/// `ctx.bind::<EventTarget, _, _>(...)`
///
/// So that the exact type of the event can be inferred.
pub struct InlineBind {
    pub invocation: TokenStream,
    pub arg: TokenTree,
}

impl Parse for InlineBind {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut invocation = TokenStream::new();

        // Must begin with an identifier, `ctx` or anything else
        let mut ident: Ident = stream.parse()?;

        invocation.push(ident);

        loop {
            invocation.push(stream.expect('.')?);

            ident = stream.parse()?;

            if ident.eq("bind") {
                invocation.push(ident);

                let arg = stream.expect('(')?;
                let _: () = stream.parse()?;

                return Ok(InlineBind { invocation, arg });
            }

            invocation.push(ident);
        }
    }
}
