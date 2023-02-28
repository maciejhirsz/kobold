use std::cell::Cell;
use std::fmt::{self, Debug};
use std::rc::Rc;

use proc_macro::{Delimiter, Span, TokenStream, TokenTree};

#[derive(Default, Debug)]
pub struct Scope {
    pub code: Vec<Code>,
    pub branches: Rc<Cell<usize>>,
}

pub enum Code {
    Segment(TokenStream),
    Html(Html),
    Nested(Nested),
}

impl Debug for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Code::Segment(segment) => write!(f, "Segment({segment})"),
            Code::Html(Html { tokens, branch, branches }) => write!(f, "Html {branch}/{}({tokens})", branches.get()),
            Code::Nested(nested) => {
                f.write_str("Nested(")?;
                nested.fmt(f)?;
                f.write_str(")")
            }
        }
    }
}

pub struct Html {
    pub tokens: TokenStream,
    pub branch: usize,
    pub branches: Rc<Cell<usize>>,
}

// #[derive(Debug)]
pub struct Nested {
    pub delimiter: Delimiter,
    pub code: Vec<Code>,
    pub span: Span,
}

impl Debug for Nested {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
