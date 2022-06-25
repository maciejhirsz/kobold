use proc_macro::{Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree};

use crate::parse::*;
use crate::parser::ParseError;

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

pub struct InlineCallback {
    pub invocation: TokenStream,
    pub arg: TokenTree,
}

impl Parse for InlineCallback {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut invocation = TokenStream::new();

        // Must begin with an identifier, `link` or anything else
        let mut ident: Ident = stream.parse()?;

        invocation.push_tt(ident);

        loop {
            if let Some(shorthand) = stream.allow_consume(':') {
                // panic!();
                invocation.write(".");
                invocation.push_tt(Ident::new("callback", shorthand.span()));

                let arg = stream.collect();
                let arg = TokenTree::Group(Group::new(Delimiter::Parenthesis, arg));

                return Ok(InlineCallback { invocation, arg });
            }

            invocation.extend([stream.expect('.')?]);

            if let Some(callback) = stream.allow_consume("callback") {
                invocation.push_tt(callback);
                break;
            }

            ident = stream.parse()?;
            invocation.push_tt(ident);
        }

        let arg = stream.expect(Delimiter::Parenthesis)?;

        stream.expect_end()?;

        Ok(InlineCallback { invocation, arg })
    }
}
