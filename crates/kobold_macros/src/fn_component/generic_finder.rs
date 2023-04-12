// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use tokens::{Ident, TokenStream, TokenTree};

use crate::parse::prelude::*;
use crate::tokenize::prelude::*;

pub enum Generic {
    Lifetime(Box<str>),
    Type(Box<str>),
}

impl Tokenize for &Generic {
    fn tokenize_in(self, stream: &mut TokenStream) {
        match self {
            Generic::Lifetime(lt) => stream.write(format_args!("'{lt},")),
            Generic::Type(ty) => stream.write(format_args!("{ty},")),
        }
    }
}

impl Parse for Generic {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let lifetime = stream.allow_consume('\'').is_some();

        let ident: Ident = stream.parse()?;

        if stream.allow_consume(':').is_some() {
            while !stream.allow(',') {
                stream.next();
            }
        }

        stream.allow_consume(',');

        let string = ident.to_string().into();

        if lifetime {
            Ok(Generic::Lifetime(string))
        } else {
            Ok(Generic::Type(string))
        }
    }
}

pub struct GenericFinder {
    generics: Vec<Generic>,
    matches: Vec<usize>,
}

impl Parse for GenericFinder {
    fn parse(stream: &mut ParseStream) -> Result<Self, ParseError> {
        let mut out = Vec::new();

        // skip opening <
        stream.next();

        loop {
            let gen = stream.parse()?;

            out.push(gen);

            if stream.allow_consume('>').is_some() {
                break;
            }
        }

        Ok(GenericFinder::new(out))
    }
}

impl GenericFinder {
    pub fn in_type(&mut self, ty: &TokenStream) -> impl Iterator<Item = &Generic> {
        self.find_inner(ty.clone());

        self.matches.drain(..).map(|idx| &self.generics[idx])
    }

    fn find_inner(&mut self, tokens: TokenStream) {
        let mut lifetime = false;

        for token in tokens {
            if token.is('\'') {
                lifetime = true;
                continue;
            }

            match token {
                TokenTree::Group(group) => self.find_inner(group.stream()),
                TokenTree::Ident(ident) => {
                    ident.with_str(|ident| {
                        for (idx, gen) in self.generics.iter().enumerate() {
                            if match (lifetime, gen) {
                                (true, Generic::Lifetime(lt)) => &**lt == ident,
                                (false, Generic::Type(ty)) => &**ty == ident,
                                _ => false,
                            } {
                                if let Err(i) = self.matches.binary_search(&idx) {
                                    self.matches.insert(i, idx);
                                }
                            }
                        }
                    });
                }
                _ => (),
            }

            lifetime = false;
        }
    }
}

impl GenericFinder {
    pub fn new(generics: Vec<Generic>) -> Self {
        let matches = Vec::with_capacity(generics.len());

        GenericFinder { generics, matches }
    }
}
