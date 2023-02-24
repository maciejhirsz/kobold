use proc_macro::{Group, Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

pub fn component(children: Option<Ident>, stream: TokenStream) -> Result<TokenStream, ParseError> {
    let mut stream = stream.parse_stream();

    let sig: FnSignature = stream.parse()?;
    let component = FnComponent::new(children, sig)?.tokenize();

    // panic!("{component}");

    Ok(component)
}

pub fn args(stream: TokenStream) -> Result<Option<Ident>, ParseError> {
    let mut stream = stream.parse_stream();

    if stream.end() {
        return Ok(None);
    }

    let mut children = match stream.expect("children")? {
        TokenTree::Ident(ident) => ident,
        _ => panic!(),
    };

    if stream.allow_consume(':').is_some() {
        children = stream.parse()?;
    }

    Ok(Some(children))
}

#[derive(Debug)]
struct FnSignature {
    name: Ident,
    arguments: Vec<Arg>,
    body: TokenTree,
}

#[derive(Debug)]
struct FnComponent {
    name: Ident,
    fields: Vec<Field>,
    render: TokenStream,
    children: Option<Field>,
}

impl FnComponent {
    fn new(children: Option<Ident>, mut sig: FnSignature) -> Result<FnComponent, ParseError> {
        let children = match &children {
            Some(children) => {
                let ident = children.to_string();

                let children_idx = sig.arguments.iter().position(|arg| arg.name.eq(&ident));

                match children_idx {
                    Some(idx) => Some(sig.arguments.remove(idx).into()),
                    None => {
                        return Err(ParseError::new(
                            format!(
                                "Missing argument `{ident}` required to capture component children"
                            ),
                            children.span(),
                        ));
                    }
                }
            }
            None => None,
        };

        let render = match sig.body {
            TokenTree::Group(group) => group.stream(),
            tt => tt.into(),
        };

        Ok(FnComponent {
            name: sig.name,
            fields: sig.arguments.into_iter().map(Into::into).collect(),
            render,
            children,
        })
    }
}

#[derive(Debug)]
struct Arg {
    name: Ident,
    ty: TokenStream,
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

impl Parse for Arg {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let name = stream.parse()?;

        stream.expect(':')?;

        let ty = stream.take_while(|token| !token.is(',')).collect();

        Ok(Arg { name, ty })
    }
}

impl From<Arg> for Field {
    fn from(arg: Arg) -> Field {
        let mut lifetime = false;

        fn substitute_lifetimes(stream: &mut ParseStream, lifetime: &mut bool) -> TokenStream {
            let mut out = TokenStream::new();

            while let Some(mut token) = stream.next() {
                if token.is(',') {
                    break;
                }

                if token.is('&') && !stream.allow('\'') {
                    *lifetime = true;

                    out.write((token, "'a"));
                    continue;
                }

                if let TokenTree::Group(g) = &mut token {
                    let mut group_stream = g.stream().parse_stream();

                    *g = Group::new(
                        g.delimiter(),
                        substitute_lifetimes(&mut group_stream, lifetime),
                    );
                }

                out.write(token);
            }

            out
        }

        let ty = substitute_lifetimes(&mut arg.ty.parse_stream(), &mut lifetime);

        Field {
            name: arg.name,
            ty,
            lifetime,
        }
    }
}

impl Tokenize for FnComponent {
    fn tokenize(self) -> TokenStream {
        let mut destruct = TokenStream::new();
        let mut field_iter = self.fields.iter();

        if let Some(first) = field_iter.next() {
            destruct.write(first.name.clone());

            for field in field_iter {
                destruct.write((',', field.name.clone()));
            }

            destruct = ("let", self.name.clone(), block(destruct), " = self;").tokenize();
        }

        let mut struct_name = self.name.tokenize();

        let (fn_render, args) = match self.children {
            Some(children) => (
                "pub fn render_with",
                group('(', ("self,", children)).tokenize(),
            ),
            None => ("pub fn render", "(self)".tokenize()),
        };
        let (imp, ret) = if self.fields.iter().any(|field| field.lifetime) {
            write!(struct_name, "<'a>");

            ("impl<'a>", "-> impl Html + 'a")
        } else {
            ("impl", "-> impl Html")
        };

        let mut out = ("struct", struct_name.clone()).tokenize();

        if self.fields.is_empty() {
            ';'.tokenize_in(&mut out);
        } else {
            block(each(self.fields)).tokenize_in(&mut out);
        };

        (
            imp,
            struct_name,
            block((fn_render, args, ret, block((destruct, self.render)))),
        )
            .tokenize_in(&mut out);

        out
    }
}

impl Tokenize for Field {
    fn tokenize(self) -> TokenStream {
        (self.name, ':', self.ty, ',').tokenize()
    }
}
