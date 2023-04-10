// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::Cell;
use std::fmt::{self, Debug};
use std::rc::Rc;

use tokens::{Delimiter, Span, TokenStream, TokenTree};

#[derive(Default, Debug)]
pub struct Scope {
    pub code: Vec<Code>,
}

pub enum Code {
    Segment(TokenStream),
    Scoped(Scoped),
    Nested(Nested),
}

impl Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Code::Segment(segment) => write!(f, "Segment({segment})"),
            Code::Scoped(scoped) => {
                f.write_str("Scoped(")?;
                scoped.fmt(f)?;
                f.write_str(")")
            }
            Code::Nested(nested) => {
                f.write_str("Nested(")?;
                nested.fmt(f)?;
                f.write_str(")")
            }
        }
    }
}

pub struct Scoped {
    pub tokens: TokenStream,
    pub span: Span,
    pub branch: u8,
    pub branches: Option<Rc<Cell<u8>>>,
}

impl Scoped {
    pub fn new(tokens: TokenStream, span: Span, branches: Option<Rc<Cell<u8>>>) -> Self {
        let branch = match branches {
            Some(ref branches) => {
                let n = branches.get();
                branches.set(n + 1);
                n
            }
            None => 0,
        };

        Scoped {
            tokens,
            span,
            branch,
            branches,
        }
    }
}

impl Debug for Scoped {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref branches) = self.branches {
            let branch = self.branch + 1;
            let branches = branches.get();

            if branches > 1 {
                write!(f, "<{branch}/{branches}> ")?;
            }
        }

        write!(f, "{}", self.tokens)
    }
}

// #[derive(Debug)]
pub struct Nested {
    pub delimiter: Delimiter,
    pub code: Vec<Code>,
    pub span: Span,
}

impl Debug for Nested {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.delimiter {
            Delimiter::Brace => f.debug_set().entries(self.code.iter()).finish(),
            Delimiter::Parenthesis => {
                let mut tup = f.debug_tuple("");

                for code in &self.code {
                    tup.field(code);
                }

                tup.finish()
            }
            _ => f.debug_list().entries(self.code.iter()).finish(),
        }
    }
}

pub struct Return {
    pub ret: TokenTree,
    pub tokens: TokenStream,
}

impl Debug for Return {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.tokens)
    }
}

#[derive(Debug)]
pub struct Branch {
    pub code: Vec<Code>,
}
