use std::ops::Deref;

use web_sys::Text;

#[cfg(feature = "stateful")]
use crate::stateful::{IntoState, Then};
use crate::{Element, Mountable, View};

pub struct ValueProduct<T> {
    value: T,
    el: Element,
}

impl<T: 'static> Mountable for ValueProduct<T> {
    type Js = Text;

    fn el(&self) -> &Element {
        &self.el
    }
}

impl View for String {
    type Product = ValueProduct<String>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(&self);

        ValueProduct { value: self, el }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self {
            p.value = self;
            p.el.set_text(&p.value);
        }
    }
}

impl View for &String {
    type Product = ValueProduct<String>;

    fn build(self) -> Self::Product {
        self.as_str().build()
    }

    fn update(self, p: &mut Self::Product) {
        View::update(self.as_str(), p)
    }
}

pub trait Stringify {
    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R;

    #[inline]
    fn no_diff(self) -> NoDiff<Self>
    where
        Self: Sized,
    {
        NoDiff(self)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NoDiff<T>(pub(crate) T);

impl<T> Deref for NoDiff<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Stringify> View for NoDiff<T> {
    type Product = Element;

    fn build(self) -> Self::Product {
        self.0.stringify(Element::new_text)
    }

    fn update(self, _: &mut Self::Product) {}
}

impl Stringify for &'static str {
    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R {
        f(self)
    }
}

impl Stringify for bool {
    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R {
        f(if *self { "true" } else { "false" })
    }
}

macro_rules! stringify_int {
    ($($t:ty),*) => {
        $(
            impl Stringify for $t {
                fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R {
                    let mut buf = itoa::Buffer::new();

                    f(buf.format(*self))
                }
            }
        )*
    };
}

macro_rules! stringify_float {
    ($($t:ty),*) => {
        $(
            impl Stringify for $t {
                fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R {
                    let mut buf = ryu::Buffer::new();

                    f(buf.format(*self))
                }
            }
        )*
    };
}

macro_rules! impl_stringify {
    ($($t:ty),*) => {
        $(
            impl View for $t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    let el = self.stringify(Element::new_text);

                    ValueProduct { value: self, el }
                }

                fn update(self, p: &mut Self::Product) {
                    if p.value != self {
                        p.value = self;

                        self.stringify(|s| p.el.set_text(s));
                    }
                }
            }

            impl View for &$t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    (*self).build()
                }

                fn update(self, p: &mut Self::Product) {
                    View::update(*self, p);
                }
            }

            impl IntoState for $t {
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

impl View for &str {
    type Product = ValueProduct<String>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self);

        ValueProduct {
            value: self.into(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self {
            self.clone_into(&mut p.value);
            p.el.set_text(self);
        }
    }
}

impl View for &&str {
    type Product = ValueProduct<String>;

    fn build(self) -> Self::Product {
        View::build(*self)
    }

    fn update(self, p: &mut Self::Product) {
        View::update(*self, p);
    }
}

#[cfg(feature = "stateful")]
impl IntoState for &str {
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
pub struct FastDiff<'a>(&'a str);

impl Deref for FastDiff<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl View for FastDiff<'_> {
    type Product = ValueProduct<usize>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self.0);

        ValueProduct {
            value: self.0.as_ptr() as usize,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self.0.as_ptr() as usize {
            p.value = self.0.as_ptr() as usize;
            p.el.set_text(self.0);
        }
    }
}

stringify_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
stringify_float!(f32, f64);

impl_stringify!(bool, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64);
