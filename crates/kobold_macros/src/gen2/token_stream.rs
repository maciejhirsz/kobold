use std::cell::RefCell;
use std::fmt::{Arguments, Write};
use std::str::FromStr;

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

use crate::parse::ParseStream;

pub trait TokenStreamExt {
    fn write(&mut self, rust: &str);

    fn push(&mut self, tt: impl Into<TokenTree>);

    fn parse_stream(self) -> ParseStream;

    fn group(self, delim: Delimiter) -> TokenStream;

    fn write_fmt(&mut self, args: Arguments);

    fn make(args: Arguments) -> Self;
}

impl TokenStreamExt for TokenStream {
    fn write(&mut self, rust: &str) {
        self.extend(TokenStream::from_str(rust).unwrap());
    }

    fn push(&mut self, tt: impl Into<TokenTree>) {
        self.extend([tt.into()]);
    }

    fn parse_stream(self) -> ParseStream {
        self.into_iter().peekable()
    }

    fn group(self, delim: Delimiter) -> TokenStream {
        TokenStream::from(TokenTree::Group(Group::new(delim, self)))
    }

    /// This allows write! macro to write to the TokenStream, auto-parsing all tokens
    fn write_fmt(&mut self, args: Arguments) {
        self.extend(Self::make(args))
    }

    /// Create a `TokenStream` from formatted `Arguments`
    fn make(args: Arguments) -> Self {
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
            let _ = buf.write_fmt(args);

            TokenStream::from_str(&buf).unwrap()
        })
    }
}
