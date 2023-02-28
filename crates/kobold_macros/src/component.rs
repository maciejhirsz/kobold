use proc_macro::{Group, Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

use crate::branching::Scope;

#[derive(Default)]
pub struct ComponentArgs {
    branching: Option<Ident>,
    children: Option<Ident>,
}

pub fn component(args: ComponentArgs, stream: TokenStream) -> Result<TokenStream, ParseError> {
    let mut stream = stream.parse_stream();

    let sig: FnSignature = stream.parse()?;
    let mut component = FnComponent::new(&args, sig)?;

    if args.branching.is_some() {
        let scope: Scope = parse(component.render)?;

        // panic!("{scope:#?}");

        component.render = scope.tokenize();
    }

    Ok(component.tokenize())
}

pub fn args(stream: TokenStream) -> Result<ComponentArgs, ParseError> {
    let mut stream = stream.parse_stream();
    let mut args = ComponentArgs::default();

    if stream.end() {
        return Ok(args);
    }

    enum Token {
        Children,
        Branching,
    }

    loop {
        let ident: Ident = stream.parse()?;

        let token = ident.with_str(|s| match s {
            "children" => Ok(Token::Children),
            "branching" => Ok(Token::Branching),
            _ => Err(ParseError::new(
                "Unknown attribute, allowed: branching, children",
                ident.span(),
            )),
        })?;

        match token {
            Token::Branching => args.branching = Some(ident),
            Token::Children => {
                args.children = Some(ident);

                if stream.allow_consume(':').is_some() {
                    args.children = Some(stream.parse()?);
                }
            }
        }

        if stream.end() {
            break;
        }

        stream.expect(',')?;

        if stream.end() {
            break;
        }
    }

    Ok(args)
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
    fn new(args: &ComponentArgs, mut sig: FnSignature) -> Result<FnComponent, ParseError> {
        let children = match &args.children {
            Some(children) => {
                let ident = children.to_string();

                let children_idx = sig.arguments.iter().position(|arg| arg.name.eq_str(&ident));

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
    fn tokenize_in(self, out: &mut TokenStream) {
        let mut destruct = TokenStream::new();
        let mut field_iter = self.fields.iter();

        if let Some(first) = field_iter.next() {
            destruct.write(first.name.clone());

            for field in field_iter {
                destruct.write((',', field.name.clone()));
            }

            destruct = ("let Self", block(destruct), " = self;").tokenize();
        }

        let name = self.name;

        let (fn_render, args) = match self.children {
            Some(children) => ("pub fn render_with", ("self,", children).tokenize()),
            None => ("pub fn render", "self".tokenize()),
        };
        let (lifetime, ret) = if self.fields.iter().any(|field| field.lifetime) {
            ("<'a>", "-> impl Html + 'a")
        } else {
            ("", "-> impl Html")
        };

        write!(out, "struct {name}{lifetime}");

        if self.fields.is_empty() {
            out.write(';');
        } else {
            out.write(block(each(self.fields)));
        };

        write!(out, "impl{lifetime} {name}{lifetime}");

        out.write(block((
            fn_render,
            group('(', args),
            ret,
            block((destruct, self.render)),
        )));
    }
}

impl Tokenize for Field {
    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.write((self.name, ':', self.ty, ','))
    }
}
