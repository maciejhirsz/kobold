use std::ops::Deref;

use crate::prelude::{IntoState, ShouldRender};
use crate::{Element, Html, Mountable};

pub struct ValueProduct<T> {
    value: T,
    el: Element,
}

impl<T: 'static> Mountable for ValueProduct<T> {
    fn el(&self) -> &Element {
        &self.el
    }
}

impl Html for String {
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

impl Html for &String {
    type Product = ValueProduct<String>;

    fn build(self) -> Self::Product {
        self.as_str().build()
    }

    fn update(self, p: &mut Self::Product) {
        Html::update(self.as_str(), p)
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

#[repr(transparent)]
pub struct NoDiff<T>(T);

impl<T> Deref for NoDiff<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Stringify> Html for NoDiff<T> {
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
            impl Html for $t {
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

            impl Html for &$t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    (*self).build()
                }

                fn update(self, p: &mut Self::Product) {
                    Html::update(*self, p);
                }
            }

            impl IntoState for $t {
                type State = Self;

                fn init(self) -> Self {
                    self
                }

                fn update(self, state: &mut Self) -> ShouldRender {
                    if *state != self {
                        *state = self;
                        ShouldRender::Yes
                    } else {
                        ShouldRender::No
                    }
                }
            }
        )*
    };
}

impl Html for &str {
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

impl Html for &&str {
    type Product = ValueProduct<String>;

    fn build(self) -> Self::Product {
        Html::build(*self)
    }

    fn update(self, p: &mut Self::Product) {
        Html::update(*self, p);
    }
}

impl IntoState for &str {
    type State = String;

    fn init(self) -> String {
        self.into()
    }

    fn update(self, state: &mut String) -> ShouldRender {
        if *state != self {
            self.clone_into(state);
            ShouldRender::Yes
        } else {
            ShouldRender::No
        }
    }
}

pub trait StrExt {
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

impl Html for FastDiff<'_> {
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
