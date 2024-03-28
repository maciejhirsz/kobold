// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `Parse` logic for different syntax elements

use std::fmt::{self, Display, Write};

use tokens::{Ident, Literal, TokenStream};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

/// Regular Rust `<Generic, Types>`, we don't care about what they are,
/// but we do care about nested angle braces.
#[derive(Clone)]
pub struct Generics {
    pub tokens: TokenStream,
}

impl Parse for Generics {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let opening = stream.expect('<')?;

        let mut depth = 1;
        let mut tokens = opening.tokenize();

        for token in stream {
            if token.is('<') {
                depth += 1;
            } else if token.is('>') {
                depth -= 1;

                if depth == 0 {
                    tokens.write(token);

                    return Ok(Generics { tokens });
                }
            }

            tokens.write(token);
        }

        Err(ParseError::new(
            "Missing closing > for generic type declaration",
            tokens.into_iter().next(),
        ))
    }
}

impl Tokenize for Generics {
    fn tokenize(self) -> TokenStream {
        self.tokens
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.extend(self.tokens);
    }
}

/// CSS-style label, matches sequences of identifiers with dashes allowed.
#[derive(Debug)]
pub struct CssLabel {
    /// Complete label with dashes
    pub label: String,
    /// Last ident in label
    pub ident: Ident,
}

impl Display for CssLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

impl Parse for CssLabel {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut ident: Ident = stream.parse()?;
        let mut label = String::new();

        // let span = ident.span();

        write!(&mut label, "{ident}").unwrap();

        while stream.allow_consume('-').is_some() {
            ident = stream.parse()?;

            write!(&mut label, "-{ident}").unwrap();
        }

        Ok(CssLabel { label, ident })
    }
}

impl CssLabel {
    pub fn into_literal(self) -> Literal {
        let mut lit = string(&self.label);

        // Keep resolution to literal, but change location
        lit.set_span(lit.span().located_at(self.ident.span()));
        lit
    }
}
