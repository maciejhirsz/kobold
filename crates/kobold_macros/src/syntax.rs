use proc_macro::{Delimiter, Ident, Literal, Span, TokenStream, TokenTree};

use crate::parse::*;

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

pub struct InlineBind {
    pub invocation: TokenStream,
    pub arg: TokenTree,
}

impl Parse for InlineBind {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut invocation = TokenStream::new();

        // Must begin with an identifier, `ctx` or anything else
        let mut ident: Ident = stream.parse()?;
        let mut done = false;

        invocation.push(ident);

        while !done {
            invocation.push(stream.expect('.')?);

            ident = stream.parse()?;

            done = ident.to_string() == "bind";

            invocation.push(ident);
        }

        let arg = stream.expect(Delimiter::Parenthesis)?;

        stream.expect_end()?;

        Ok(InlineBind { invocation, arg })
    }
}
