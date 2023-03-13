use proc_macro::{Group, TokenStream, TokenTree};

use crate::tokenize::prelude::*;

use super::ast::{Code, Nested, Scope, Scoped};

impl Tokenize for Scope {
    fn tokenize_in(self, stream: &mut TokenStream) {
        self.code.tokenize_in(stream);
    }
}

impl Tokenize for Vec<Code> {
    fn tokenize_in(self, stream: &mut TokenStream) {
        for code in self {
            code.tokenize_in(stream);
        }
    }
}

impl Tokenize for Code {
    fn tokenize_in(self, stream: &mut TokenStream) {
        match self {
            Code::Segment(segment) => stream.extend(segment),
            Code::Scoped(scoped) => scoped.tokenize_in(stream),
            Code::Nested(nested) => nested.tokenize_in(stream),
        }
    }
}

impl Tokenize for Scoped {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let branches = self.branches.map(|b| b.get()).unwrap_or(0);

        if branches > 1 {
            let variant = [b'A' + self.branch];
            let variant = std::str::from_utf8(&variant).unwrap();

            let branch = format!("::kobold::branching::Branch{branches}::{variant}").tokenize();

            stream.extend(branch.into_iter().map(|mut tt| {
                match &mut tt {
                    TokenTree::Ident(ident) => ident.set_span(self.span),
                    TokenTree::Punct(punct) => punct.set_span(self.span),
                    _ => (),
                }

                tt
            }));

            let mut group = group('(', self.tokens);
            group.set_span(self.span);

            stream.write(group);
            return;
        }

        stream.extend(self.tokens);
    }
}

impl Tokenize for Nested {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let mut group = Group::new(self.delimiter, self.code.tokenize());

        group.set_span(self.span);

        stream.write(group)
    }
}
