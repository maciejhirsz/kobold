use proc_macro::{Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

pub fn fn_component(stream: TokenStream) -> Result<TokenStream, ParseError> {
    let mut stream = stream.parse_stream();

    let sig: FnSignature = stream.parse()?;
    let component = FnComponent::from(sig).tokenize();

    // panic!("{component}");

    Ok(component)
}

#[derive(Debug)]
struct FnSignature {
    name: Ident,
    arguments: Vec<Field>,
    body: TokenTree,
}

#[derive(Debug)]
struct FnComponent {
    name: Ident,
    fields: Vec<Field>,
    render: TokenTree,
}

impl From<FnSignature> for FnComponent {
    fn from(sig: FnSignature) -> FnComponent {
        FnComponent {
            name: sig.name,
            fields: sig.arguments,
            render: sig.body,
        }
    }
}

#[derive(Debug)]
struct Field {
    name: Ident,
    ty: TokenStream,
    lifetime: bool,
}

impl Parse for FnSignature {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        stream.expect("fn")?;

        let name = stream.parse()?;
        let mut arguments = Vec::new();

        if let TokenTree::Group(args) = stream.expect('(')? {
            let mut stream = args.stream().parse_stream();

            while !stream.end() {
                arguments.push(stream.parse()?);
            }
        }

        stream.expect('-')?;
        stream.expect('>')?;
        stream.expect("impl")?;
        stream.expect("Html")?;

        let body = stream.expect('{')?;

        Ok(FnSignature {
            name,
            arguments,
            body,
        })
    }
}

impl Parse for Field {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let name = stream.parse()?;
        let mut ty = TokenStream::new();
        let mut lifetime = false;

        stream.expect(':')?;

        while let Some(token) = stream.next() {
            if token.is(',') {
                break;
            }

            if token.is('&') {
                lifetime = true;

                ty.write((token, "'a"));
                continue;
            }

            ty.write(token);
        }

        Ok(Field { name, ty, lifetime })
    }
}

impl Tokenize for FnComponent {
    fn tokenize(self) -> TokenStream {
        let mut destruct = TokenStream::new();

        for field in &self.fields {
            write!(&mut destruct, "let {} = self.{};", field.name, field.name);
        }

        let mut struct_name = self.name.tokenize();

        let (imp, sig) = if self.fields.iter().any(|field| field.lifetime) {
            write!(struct_name, "<'a>");

            ("impl<'a>", "fn render(self) -> impl Html + 'a")
        } else {
            ("impl", "fn render(self) -> impl Html")
        };

        let mut out = ("struct", struct_name.clone()).tokenize();

        if self.fields.is_empty() {
            ';'.tokenize_in(&mut out);
        } else {
            block(each(self.fields)).tokenize_in(&mut out);
        };

        (imp, struct_name, block((sig, block((destruct, self.render))))).tokenize_in(&mut out);

        out
    }
}

impl Tokenize for Field {
    fn tokenize(self) -> TokenStream {
        (self.name, ':', self.ty, ',').tokenize()
    }
}
