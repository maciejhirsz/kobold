// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::fmt::{Arguments, Write};
use std::str::FromStr;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::parse::ParseStream;

pub mod prelude {
    pub use super::{block, call, each, group, ident, string, TokenStreamExt, Tokenize};
}

pub fn group(delim: char, tokens: impl Tokenize) -> Group {
    let delim = match delim {
        '{' => Delimiter::Brace,
        '[' => Delimiter::Bracket,
        '(' => Delimiter::Parenthesis,
        _ => panic!("Invalid delimiter {delim}"),
    };
    Group::new(delim, tokens.tokenize())
}

pub fn call(path: impl Tokenize, tokens: impl Tokenize) -> TokenStream {
    (path, group('(', tokens)).tokenize()
}

pub fn block(tokens: impl Tokenize) -> impl Tokenize {
    group('{', tokens)
}

pub fn string(lit: &str) -> Literal {
    Literal::string(lit)
}

pub fn ident(ident: &str) -> Ident {
    Ident::new(ident, Span::call_site())
}

pub fn each<I>(iter: I) -> TokenizeIter<I> {
    TokenizeIter(iter)
}

pub trait Tokenize: Sized {
    fn tokenize(self) -> TokenStream {
        let mut stream = TokenStream::new();

        self.tokenize_in(&mut stream);

        stream
    }

    fn tokenize_in(self, stream: &mut TokenStream);
}

impl Tokenize for TokenStream {
    fn tokenize(self) -> TokenStream {
        self
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.extend(self)
    }
}

impl Tokenize for ParseStream {
    fn tokenize(self) -> TokenStream {
        self.collect()
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.extend(self);
    }
}

impl<T: Tokenize> Tokenize for Option<T> {
    fn tokenize_in(self, stream: &mut TokenStream) {
        if let Some(item) = self {
            item.tokenize_in(stream)
        }
    }
}

impl<T: Tokenize + Clone> Tokenize for &T {
    fn tokenize(self) -> TokenStream {
        self.clone().tokenize()
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        self.clone().tokenize_in(stream);
    }
}

pub struct TokenizeIter<I>(I);

impl Tokenize for () {
    fn tokenize_in(self, _: &mut TokenStream) {}
}

impl<I> Tokenize for TokenizeIter<I>
where
    I: IntoIterator,
    I::Item: Tokenize,
{
    fn tokenize(self) -> TokenStream {
        let mut iter = self.0.into_iter();

        let mut stream = match iter.next() {
            Some(item) => item.tokenize(),
            None => return TokenStream::new(),
        };

        for item in iter {
            item.tokenize_in(&mut stream);
        }

        stream
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        for item in self.0 {
            item.tokenize_in(stream);
        }
    }
}

impl Tokenize for Arguments<'_> {
    fn tokenize(self) -> TokenStream {
        thread_local! {
            // We need to collect the whole args to a string first, and then write them to stream
            // all at once, otherwise TokenStream::write might fail if there are any Groups
            // inside the interlaced string.
            //
            // To avoid allocating each time, we use a thread local buffer
            static TOKEN_STREAM_BUF: RefCell<String> = RefCell::new(String::with_capacity(128));
        }

        TOKEN_STREAM_BUF.with(move |buf| {
            let mut buf = buf.borrow_mut();

            buf.clear();

            // Writing to String is infallible
            let _ = buf.write_fmt(self);

            TokenStream::from_str(&buf).unwrap()
        })
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.extend(self.tokenize())
    }
}

impl Tokenize for &str {
    fn tokenize(self) -> TokenStream {
        TokenStream::from_str(self).unwrap()
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.extend(self.tokenize())
    }
}

impl Tokenize for char {
    fn tokenize(self) -> TokenStream {
        Punct::new(self, Spacing::Alone).tokenize()
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        Punct::new(self, Spacing::Alone).tokenize_in(stream);
    }
}

impl Tokenize for TokenTree {
    fn tokenize(self) -> TokenStream {
        TokenStream::from(self)
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        stream.extend([self])
    }
}

macro_rules! impl_tt {
    ($($typ:ident),*) => {
        $(
            impl Tokenize for $typ {
                fn tokenize(self) -> TokenStream {
                    TokenStream::from(TokenTree::$typ(self))
                }

                fn tokenize_in(self, stream: &mut TokenStream) {
                    stream.extend([TokenTree::$typ(self)])
                }
            }
        )*
    };
}

impl_tt!(Literal, Ident, Punct, Group);

macro_rules! impl_tuple {
    (A: 0, $($t:ident: $n:tt),*) => {
        impl<A, $($t,)*> Tokenize for (A, $($t,)*)
        where
            A: Tokenize,
            $($t: Tokenize,)*
        {
            fn tokenize(self) -> TokenStream {
                let mut stream = self.0.tokenize();
                $(self.$n.tokenize_in(&mut stream);)*
                stream
            }

            fn tokenize_in(self, stream: &mut TokenStream)
            where
                Self: Sized,
            {
                self.0.tokenize_in(stream);
                $(self.$n.tokenize_in(stream);)*
            }
        }
    };
}

impl_tuple!(A: 0, B: 1);
impl_tuple!(A: 0, B: 1, C: 2);
impl_tuple!(A: 0, B: 1, C: 2, D: 3);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_tuple!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);

pub trait TokenStreamExt {
    fn write<T: Tokenize>(&mut self, tokens: T);

    fn parse_stream(self) -> ParseStream;

    fn write_fmt(&mut self, args: Arguments);
}

impl TokenStreamExt for TokenStream {
    fn write<T: Tokenize>(&mut self, tokens: T) {
        tokens.tokenize_in(self);
    }

    fn parse_stream(self) -> ParseStream {
        self.into_iter().peekable()
    }

    /// This allows write! macro to write to the TokenStream, auto-parsing all tokens
    fn write_fmt(&mut self, args: Arguments) {
        self.extend(args.tokenize())
    }
}
