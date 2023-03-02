use proc_macro::{Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

use crate::branching::Scope;
use crate::syntax::Generics;

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
        AutoBranch,
    }

    loop {
        let ident: Ident = stream.parse()?;

        let token = ident.with_str(|s| match s {
            "children" => Ok(Token::Children),
            "auto_branch" => Ok(Token::AutoBranch),
            _ => Err(ParseError::new(
                "Unknown attribute, allowed: auto_branch, children",
                ident.span(),
            )),
        })?;

        match token {
            Token::AutoBranch => args.branching = Some(ident),
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

struct FnSignature {
    name: Ident,
    generics: Option<Generics>,
    arguments: Vec<Arg>,
    ret: TokenStream,
    body: TokenTree,
}

struct FnComponent {
    name: Ident,
    generics: Option<Generics>,
    arguments: Vec<Arg>,
    ret: TokenStream,
    render: TokenStream,
    children: Option<Arg>,
}

impl FnComponent {
    fn new(args: &ComponentArgs, mut sig: FnSignature) -> Result<FnComponent, ParseError> {
        let children = match &args.children {
            Some(children) => {
                let ident = children.to_string();

                let children_idx = sig.arguments.iter().position(|arg| arg.name.eq_str(&ident));

                match children_idx {
                    Some(idx) => Some(sig.arguments.remove(idx)),
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
            generics: sig.generics,
            arguments: sig.arguments,
            ret: sig.ret,
            render,
            children,
        })
    }
}

struct Arg {
    name: Ident,
    ty: TokenStream,
}

impl Parse for FnSignature {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        stream.expect("fn")?;

        let name = stream.parse()?;

        let generics = if stream.allow('<') {
            Some(stream.parse()?)
        } else {
            None
        };

        let mut arguments = Vec::new();

        if let TokenTree::Group(args) = stream.expect('(')? {
            let mut stream = args.stream().parse_stream();

            while !stream.end() {
                arguments.push(stream.parse()?);
            }
        }

        let mut body = None;

        let ret = stream
            .map_while(|tt| {
                if tt.is('{') {
                    body = Some(tt);

                    None
                } else {
                    Some(tt)
                }
            })
            .collect();

        match body {
            Some(body) => Ok(FnSignature {
                name,
                generics,
                arguments,
                ret,
                body,
            }),
            None => Err(ParseError::new("Missing body for function", name.span())),
        }
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

impl Tokenize for FnComponent {
    fn tokenize_in(self, out: &mut TokenStream) {
        let name = &self.name;

        let mut args = if self.arguments.is_empty() {
            ("_:", name).tokenize()
        } else {
            let destruct = (name, block(each(self.arguments.iter().map(Arg::name))));
            let props_ty = (name, '<', each(self.arguments.iter().map(Arg::ty)), '>');

            (destruct, ':', props_ty).tokenize()
        };

        let fn_render = match self.children {
            Some(children) => {
                args.write((',', children));
                "pub fn render_with"
            }
            None => "pub fn render",
        };

        out.write(("#[allow(non_camel_case_types)]", "struct", name));

        if self.arguments.is_empty() {
            out.write(';');
        } else {
            out.write((
                '<',
                each(self.arguments.iter().map(Arg::generic)).tokenize(),
                '>',
                block(each(self.arguments.iter().map(Arg::field))),
            ));
        };

        out.write(("impl", name));

        out.write(block((
            fn_render,
            self.generics,
            group('(', args),
            self.ret,
            block(self.render),
        )));
    }
}

impl Arg {
    fn ty(&self) -> impl Tokenize + '_ {
        (&self.ty, ',')
    }

    fn name(&self) -> impl Tokenize + '_ {
        (&self.name, ',')
    }

    fn generic(&self) -> impl Tokenize + '_ {
        (&self.name, "=(),")
    }

    fn field(&self) -> impl Tokenize + '_ {
        (&self.name, ':', &self.name, ',')
    }
}

impl Tokenize for Arg {
    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.write((self.name, ':', self.ty, ','))
    }
}
