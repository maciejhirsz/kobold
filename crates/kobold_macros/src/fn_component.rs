// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Write;

use tokens::{Group, Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

use crate::branching::Scope;
use crate::syntax::Generics;

mod generic_finder;

use generic_finder::GenericFinder;

#[derive(Default)]
pub struct ComponentArgs {
    branching: Option<Ident>,
    children: Option<Ident>,
    defaults: Vec<(Ident, Value)>,
}

enum Value {
    Default,
    Expr(TokenStream),
}

pub fn component(mut args: ComponentArgs, stream: TokenStream) -> Result<TokenStream, ParseError> {
    let mut stream = stream.parse_stream();

    let sig: Function = stream.parse()?;
    let mut component = FnComponent::new(&mut args, sig)?;

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
        Default,
    }

    loop {
        let ident: Ident = stream.parse()?;

        let token = if stream.allow_consume('?').is_some() {
            Token::Default
        } else {
            ident.with_str(|s| match s {
                "children" => Ok(Token::Children),
                "auto_branch" => Ok(Token::AutoBranch),
                _ => Err(ParseError::new(
                    "Unknown attribute, allowed: `auto_branch`, `children`, or `<parameter>?`",
                    ident.span(),
                )),
            })?
        };

        match token {
            Token::AutoBranch => args.branching = Some(ident),
            Token::Children => {
                args.children = Some(ident);

                if stream.allow_consume(':').is_some() {
                    args.children = Some(stream.parse()?);
                }
            }
            Token::Default => {
                let value = if stream.allow_consume(':').is_some() {
                    let mut value = TokenStream::new();

                    while let Some(tt) = stream.peek() {
                        if tt.is(',') {
                            break;
                        }

                        value.extend(stream.next());
                    }

                    Value::Expr(value)
                } else {
                    Value::Default
                };

                args.defaults.push((ident, value));
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

struct Function {
    r#fn: TokenTree,
    r#pub: Option<TokenStream>,
    name: Ident,
    generics: Option<Generics>,
    raw_args: Option<Group>,
    arguments: Vec<Argument>,
    r#return: TokenStream,
    body: TokenTree,
}

struct FnComponent {
    r#fn: TokenTree,
    r#mod: Ident,
    r#pub: Option<TokenStream>,
    name: Ident,
    generics: Option<Generics>,
    raw_args: Option<Group>,
    arguments: Vec<Argument>,
    ret: TokenStream,
    render: TokenStream,
}

impl FnComponent {
    fn new(args: &mut ComponentArgs, mut fun: Function) -> Result<FnComponent, ParseError> {
        if let Some(children) = args.children.take() {
            let ident = children.to_string();
            let mut found = false;

            for arg in fun.arguments.iter_mut() {
                if arg.name.eq_str(&ident) {
                    arg.name = Ident::new("children", arg.name.span());

                    found = true;
                    break;
                }
            }

            if !found {
                return Err(ParseError::new(
                    format!(
                        "Missing argument `{ident}` required to capture component children"
                    ),
                    children.span(),
                ));
            }
        }

        let mut temp_var = String::with_capacity(40);

        'outer: for (var, value) in args.defaults.drain(..) {
            temp_var.clear();

            let _ = write!(temp_var, "{var}");

            for arg in fun.arguments.iter_mut() {
                if arg.name.eq_str(&temp_var) {
                    arg.default = Some(value);
                    continue 'outer;
                }
            }

            return Err(ParseError::new(
                format!("Parameter `{var}` missing in the component `{}`", fun.name),
                var.span(),
            ));
        }

        let render = match fun.body {
            TokenTree::Group(group) => group.stream(),
            tt => tt.into(),
        };

        let r#mod = Ident::new("mod", fun.r#fn.span());

        Ok(FnComponent {
            r#fn: fun.r#fn,
            r#mod,
            r#pub: fun.r#pub,
            name: fun.name,
            generics: fun.generics,
            raw_args: fun.raw_args,
            arguments: fun.arguments,
            ret: fun.r#return,
            render,
        })
    }
}

struct Argument {
    name: Ident,
    ty: TokenStream,
    default: Option<Value>,
}

impl Parse for Function {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let r#pub = stream.allow_consume("pub").map(|tt| {
            let mut public = TokenStream::from(tt);
            public.extend(stream.allow_consume('('));
            public
        });

        let r#fn = stream.expect("fn")?;
        let name = stream.parse()?;

        let generics = if stream.allow('<') {
            Some(stream.parse()?)
        } else {
            None
        };

        let mut arguments = Vec::new();
        let mut raw_args = None;

        if let TokenTree::Group(args) = stream.expect('(')? {
            raw_args = Some(args.clone());
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
            Some(body) => Ok(Function {
                r#fn,
                r#pub,
                name,
                generics,
                raw_args,
                arguments,
                r#return: ret,
                body,
            }),
            None => Err(ParseError::new("Missing body for function", name.span())),
        }
    }
}

impl Parse for Argument {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let name = stream.parse()?;

        stream.expect(':')?;

        let ty = stream.take_while(|token| !token.is(',')).collect();

        Ok(Argument {
            name,
            ty,
            default: None,
        })
    }
}

impl Tokenize for FnComponent {
    fn tokenize_in(self, out: &mut TokenStream) {
        let mut mo = TokenStream::new();
        let name = &self.name;

        let mut finder: Option<GenericFinder> = match &self.generics {
            None => None,
            Some(generics) => generics.tokens.clone().parse_stream().parse().ok(),
        };

        let args = if self.arguments.is_empty() {
            "_: Props".tokenize()
        } else {
            let destruct = (
                "Props",
                block(each(self.arguments.iter().map(Argument::name))),
            );
            let props_ty = ("Props<", each(self.arguments.iter().map(Argument::ty)), '>');

            (destruct, ':', props_ty).tokenize()
        };

        mo.write("#[allow(non_camel_case_types)] pub struct Props");

        if self.arguments.is_empty() {
            mo.write(';');
        } else {
            mo.write((
                '<',
                each(self.arguments.iter().map(Argument::generic)).tokenize(),
                '>',
                block(each(self.arguments.iter().map(Argument::field))),
            ));
        };

        let fn_render = (
            "pub fn render",
            self.generics.clone(),
            group('(', args),
            self.ret.clone(),
            block((
                each(self.arguments.iter().map(Argument::maybe)),
                call(
                    ("super::", name),
                    each(self.arguments.iter().map(Argument::name)),
                ),
            )),
        );

        let fn_props = (
            "pub const fn props() -> Props",
            block((
                "Props",
                block(each(self.arguments.iter().map(Argument::default)).tokenize()),
            )),
        );

        mo.write((fn_props, fn_render));

        let field_generics = ('<', each(self.arguments.iter().map(Argument::name)), '>').tokenize();

        mo.write((
            "#[allow(non_camel_case_types)] impl",
            field_generics.clone(),
            "Props",
            field_generics,
            block(each(
                self.arguments
                    .iter()
                    .enumerate()
                    .map(|(i, a)| a.setter(finder.as_mut(), i, &self.arguments)),
            )),
        ));

        // panic!("{mo}");

        out.write((&self.r#pub, self.r#fn, name, self.generics, self.raw_args));
        out.write((self.ret, block(self.render)));

        out.write((
            format!("#[doc = \"`#[component]` handlers for the [`{name}`](fn.{name}.html) function.\"]").as_str(),
            self.r#pub,
            self.r#mod,
            name,
            block(("use super::*;", mo)),
        ));
    }
}

impl Argument {
    fn ty(&self) -> impl Tokenize + '_ {
        tok_fn(|stream| {
            if self.default.is_some() {
                stream.write("impl ::kobold::maybe::Maybe<");
                stream.write(&self.ty);
                stream.write('>');
            } else {
                stream.write(&self.ty);
            }

            stream.write(',');
        })
    }

    fn name(&self) -> impl Tokenize + '_ {
        (&self.name, ',')
    }

    fn generic(&self) -> impl Tokenize + '_ {
        (&self.name, "= ::kobold::maybe::Undefined,")
    }

    fn setter<'a>(
        &'a self,
        finder: Option<&mut GenericFinder>,
        pos: usize,
        args: &'a [Argument],
    ) -> impl Tokenize + 'a {
        let mut ret_generics = TokenStream::new();
        let mut body = TokenStream::new();

        let maybe_generic = self.default.is_some().then_some("Maybe");

        let where_clause = tok_fn(|stream| {
            if self.default.is_some() {
                stream.write("where Maybe: ::kobold::maybe::Maybe<");
                stream.write(&self.ty);
                stream.write('>');
            }
        });

        let maybe_ty = tok_fn(|stream| match self.default {
            Some(_) => stream.write("Maybe"),
            None => stream.write(&self.ty),
        });

        for (i, arg) in args.iter().enumerate() {
            if i == pos {
                body.write((&self.name, ":value,"));
                if self.default.is_some() {
                    ret_generics.write("Maybe,");
                } else {
                    ret_generics.write((&self.ty, ','));
                }
            } else {
                body.write((&arg.name, ":self.", &arg.name, ','));
                ret_generics.write((&arg.name, ','));
            }
        }

        let ret_type = ("-> Props<", ret_generics, '>', where_clause);

        (
            "#[inline(always)] pub fn ",
            call(
                (
                    &self.name,
                    '<',
                    finder.map(|finder| each(finder.in_type(&self.ty))),
                    maybe_generic,
                    '>',
                ),
                ("self, value:", maybe_ty),
            ),
            ret_type,
            block(("Props", block(body))),
        )
    }

    fn maybe(&self) -> impl Tokenize + '_ {
        tok_fn(|stream| {
            if let Some(value) = &self.default {
                stream.write(("let", &self.name, "=", &self.name));

                match value {
                    Value::Default => stream.write(".maybe_or(Default::default);"),
                    Value::Expr(expr) => stream.write((call(".maybe_or", ("||", expr)), ';')),
                }
            }
        })
    }

    fn default(&self) -> impl Tokenize + '_ {
        (&self.name, ": ::kobold::maybe::Undefined,")
    }

    fn field(&self) -> impl Tokenize + '_ {
        (&self.name, ':', &self.name, ',')
    }
}

impl Tokenize for Argument {
    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.write((self.name, ':', self.ty, ','))
    }
}
