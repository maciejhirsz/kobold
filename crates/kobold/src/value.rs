use std::ops::Deref;

use crate::diff::Diff;
use crate::dom::{LargeInt, Text};
#[cfg(feature = "stateful")]
use crate::stateful::{self, Then};

use crate::{Element, Mountable, View};

pub struct TextProduct<S> {
    state: S,
    el: Element,
}

impl<S: 'static> Mountable for TextProduct<S> {
    type Js = web_sys::Text;

    fn el(&self) -> &Element {
        &self.el
    }
}

impl View for String {
    type Product = TextProduct<String>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self.as_str());

        TextProduct { state: self, el }
    }

    fn update(self, p: &mut Self::Product) {
        if p.state != self {
            p.state = self;
            p.el.set_text(p.state.as_str());
        }
    }
}

impl View for &String {
    type Product = TextProduct<String>;

    fn build(self) -> Self::Product {
        self.as_str().build()
    }

    fn update(self, p: &mut Self::Product) {
        View::update(self.as_str(), p)
    }
}

impl View for &str {
    type Product = TextProduct<String>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self);

        TextProduct {
            state: self.into(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if p.state != self {
            self.clone_into(&mut p.state);
            p.el.set_text(self);
        }
    }
}

impl View for &&str {
    type Product = TextProduct<String>;

    fn build(self) -> Self::Product {
        View::build(*self)
    }

    fn update(self, p: &mut Self::Product) {
        View::update(*self, p);
    }
}

#[cfg(feature = "stateful")]
impl stateful::IntoState for &str {
    type State = String;

    fn init(self) -> String {
        self.into()
    }

    fn update(self, state: &mut String) -> Then {
        if *state != self {
            self.clone_into(state);
            Then::Render
        } else {
            Then::Stop
        }
    }
}

pub trait StrExt {
    /// Wraps a `&str` into [`FastDiff`](FastDiff).
    ///
    ///`FastDiff`'s [`View`](crate::View) implementation never allocates
    /// and only performs a fast pointer address diffing. This can lead to
    /// situations where the data behind the pointer has changed, but the
    /// view is not updated on render, hence this behavior is not default.
    ///
    /// In situations where you are sure the strings are never mutated in
    /// buffer but rather replaced (either by new allocations or from new
    /// `&'static str` slices) using `fast_diff` will improve overall
    /// runtime performance.
    fn fast_diff(&self) -> FastDiff<'_>;
}

impl StrExt for str {
    fn fast_diff(&self) -> FastDiff<'_> {
        FastDiff(self)
    }
}

#[repr(transparent)]
pub struct FastDiff<'a>(pub(crate) &'a str);

impl AsRef<str> for FastDiff<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Deref for FastDiff<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl View for FastDiff<'_> {
    type Product = TextProduct<usize>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self.0);

        TextProduct {
            state: self.0.as_ptr() as usize,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if p.state != self.0.as_ptr() as usize {
            p.state = self.0.as_ptr() as usize;
            p.el.set_text(self.0);
        }
    }
}

macro_rules! large_int {
    ($($t:ty > $d:ty),*) => {
        $(
            impl LargeInt for $t {
                type Downcast = $d;

                fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R {
                    let mut buf = itoa::Buffer::new();

                    f(buf.format(*self))
                }
            }
        )*
    };
}

macro_rules! impl_text {
    ($($t:ty),*) => {
        $(
            impl View for $t {
                type Product = TextProduct<<$t as Diff>::State>;

                fn build(self) -> Self::Product {
                    let el = Element::new(self.to_text());
                    let state = self.init();

                    TextProduct { state, el }
                }

                fn update(self, p: &mut Self::Product) {
                    if p.state != self {
                        p.state = self;
                        p.el.set_text(self);
                    }
                }
            }

            impl View for &$t {
                type Product = TextProduct<$t>;

                fn build(self) -> Self::Product {
                    (*self).build()
                }

                fn update(self, p: &mut Self::Product) {
                    View::update(*self, p);
                }
            }

            #[cfg(feature = "stateful")]
            impl stateful::IntoState for $t {
                type State = Self;

                fn init(self) -> Self {
                    self
                }

                fn update(self, state: &mut Self) -> Then {
                    if *state != self {
                        *state = self;
                        Then::Render
                    } else {
                        Then::Stop
                    }
                }
            }
        )*
    };
}

large_int!(u64 > u32, u128 > u32, i64 > i32, i128 > i32);
impl_text!(bool, u8, u16, u32, u64, u128, usize, isize, i8, i16, i32, i64, i128, f32, f64);
