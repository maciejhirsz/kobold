use proc_macro::{TokenStream, Group};

use crate::tokenize::prelude::*;

use super::ast::{Scope, Code, Html, Nested};

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
            Code::Segment(segment) => segment.tokenize_in(stream),
            Code::Html(html) => html.tokenize_in(stream),
            Code::Nested(nested) => nested.tokenize_in(stream),
        }
    }
}

impl Tokenize for Html {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let branches = self.branches.map(|b| b.get()).unwrap_or(0);

        if branches > 1 {
            let variant = [b'A' + self.branch];
            let variant = std::str::from_utf8(&variant).unwrap();

            write!(stream, "::kobold::branching::Branch{branches}::{variant}");

            stream.write(group('(', self.tokens));
            return;
        }

        stream.extend(self.tokens);
    }
}

impl Tokenize for Nested {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let group = Group::new(self.delimiter, self.code.tokenize());

        stream.write(group)
    }
}
