//! `Parse` logic for different syntax elements

use std::fmt::{self, Display, Write};

use proc_macro::{Delimiter, Ident, Literal, Spacing, Span, TokenStream, TokenTree};

use crate::parse::*;

/// Tag name for an element, either HTML element such as `div`, or a component `Foo`.
#[derive(Debug)]
pub enum TagName {
    HtmlElement {
        name: String,
        span: Span,
    },
    Component {
        name: String,
        span: Span,
        path: TokenStream,
        generics: Option<TokenStream>,
    },
}

impl PartialEq for TagName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TagName::HtmlElement { name: l, .. }, TagName::HtmlElement { name: r, .. }) => l == r,
            (TagName::Component { name: l, .. }, TagName::Component { name: r, .. }) => l == r,
            _ => false,
        }
    }
}

impl TagName {
    pub fn is_component(&self) -> bool {
        match self {
            TagName::HtmlElement { .. } => false,
            TagName::Component { .. } => true,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            TagName::HtmlElement { span, .. } => span.clone(),
            TagName::Component { span, .. } => span.clone(),
        }
    }
}

impl IntoSpan for TagName {
    fn into_span(self) -> Span {
        self.span()
    }
}

impl Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TagName::HtmlElement { name, .. } => name,
            TagName::Component { name, .. } => name,
        };

        f.write_str(name)
    }
}

impl Parse for TagName {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut ident: Ident = stream.parse()?;
        let mut name = ident.to_string();
        let mut span = ident.span();

        if name.as_bytes()[0].is_ascii_lowercase() && !stream.allow((':', Spacing::Joint)) {
            return Ok(TagName::HtmlElement { name, span });
        }

        let mut path = TokenStream::new();

        path.push(ident);

        while let Some(colon) = stream.allow_consume((':', Spacing::Joint)) {
            path.push(colon);
            path.push(stream.expect(':')?);

            ident = stream.parse()?;
            span = ident.span();

            write!(&mut name, "::{ident}").unwrap();

            path.push(ident);
        }

        let mut generics = None;

        if stream.allow('<') {
            generics = Some(Generics::parse(stream)?.tokens);
        }

        Ok(TagName::Component {
            name,
            span,
            path,
            generics,
        })
    }
}

/// Describes nesting behavior of a tag
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TagNesting {
    /// Opening tag `<...>`
    Opening,
    /// Closing tag `</...>`
    Closing,
    /// Self-closing tag `<.../>`
    SelfClosing,
}

/// Non-descript tag
#[derive(Debug)]
pub struct Tag {
    pub name: TagName,
    pub nesting: TagNesting,
    pub content: TokenStream,
}

impl Tag {
    pub fn is_closing(&self, opening: &TagName) -> bool {
        self.nesting == TagNesting::Closing && &self.name == opening
    }
}

impl Parse for Tag {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        stream.expect('<')?;

        let mut nesting = match stream.allow_consume('/') {
            Some(_) => {
                let name = stream.parse()?;

                stream.expect('>')?;

                return Ok(Tag {
                   name,
                   nesting: TagNesting::Closing,
                   content: TokenStream::new(),
                })
            },
            None => TagNesting::Opening,
        };

        let name = stream.parse()?;

        let mut content = TokenStream::new();

        while let Some(tt) = stream.next() {
            if tt.is('/') {
                if nesting == TagNesting::Opening {
                    nesting = TagNesting::SelfClosing;

                    stream.expect('>')?;

                    return Ok(Tag {
                        name,
                        nesting,
                        content,
                    });
                } else {
                    return Err(ParseError::new("Unexpected closing slash", tt));
                }
            }

            if tt.is('>') {
                return Ok(Tag {
                    name,
                    nesting,
                    content,
                });
            }

            content.push(tt);
        }

        Err(ParseError::new(format!("Missing closing > for {name}"), name))
    }
}

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

            if ident.str_eq("bind") {
                invocation.push(ident);

                let arg = stream.expect(Delimiter::Parenthesis)?;
                let _: () = stream.parse()?;

                return Ok(InlineBind { invocation, arg })
            }

            invocation.push(ident);
        }
    }
}
