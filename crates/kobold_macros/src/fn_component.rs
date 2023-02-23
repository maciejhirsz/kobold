use proc_macro::{Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

pub fn fn_component(stream: TokenStream) -> Result<TokenStream, ParseError> {
    let mut stream = stream.parse_stream();

    let sig: FnSignature = stream.parse()?;
    let component = FnComponent::from(sig).tokenize();

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

        stream.expect(':')?;

        for token in stream {
            if token.is(',') {
                break;
            }

            ty.write(token);
        }

        Ok(Field { name, ty })
    }
}

impl Tokenize for FnComponent {
    fn tokenize(self) -> TokenStream {
        let mut destruct = TokenStream::new();

        for field in &self.fields {
            write!(&mut destruct, "let {} = self.{};", field.name, field.name);
        }

        (
            "struct",
            self.name.clone(),
            block(each(self.fields)),
            "impl",
            self.name,
            block(("fn render(self) -> impl Html", block((destruct, self.render)))),
        )
            .tokenize()
    }
}

impl Tokenize for Field {
    fn tokenize(self) -> TokenStream {
        (self.name, ':', self.ty, ',').tokenize()
    }
}
