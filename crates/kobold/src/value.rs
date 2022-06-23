use crate::{Element, Html, Mountable};
use std::str;

pub struct ValueProduct<T> {
    value: T,
    el: Element,
}

impl<T: 'static> Mountable for ValueProduct<T> {
    fn el(&self) -> &Element {
        &self.el
    }
}

pub trait Stringify {
    fn stringify<F: FnOnce(&str) -> R, R>(self, f: F) -> R;
}

impl Stringify for &'static str {
    fn stringify<F: FnOnce(&str) -> R, R>(self, f: F) -> R {
        f(self)
    }
}

impl Stringify for bool {
    fn stringify<F: FnOnce(&str) -> R, R>(self, f: F) -> R {
        f(if self { "true" } else { "false" })
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
        let el = Element::new_text(self);

        ValueProduct {
            value: self.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if &p.value != self {
            p.value.clone_from(self);
            p.el.set_text(&p.value);
        }
    }
}

macro_rules! stringify_int {
    ($($t:ty),*) => {
        $(
            impl Stringify for $t {
                fn stringify<F: FnOnce(&str) -> R, R>(self, f: F) -> R {
                    let mut buf = itoa::Buffer::new();

                    f(buf.format(self))
                }
            }
        )*
    };
}

macro_rules! stringify_float {
    ($($t:ty),*) => {
        $(
            impl Stringify for $t {
                fn stringify<F: FnOnce(&str) -> R, R>(self, f: F) -> R {
                    let mut buf = ryu::Buffer::new();

                    f(buf.format(self))
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

            impl_ref_copy!($t);
        )*
    };
}

macro_rules! impl_ref_copy {
    ($t:ty) => {
        impl Html for &$t {
            type Product = ValueProduct<$t>;

            fn build(self) -> Self::Product {
                (*self).build()
            }

            fn update(self, p: &mut Self::Product) {
                (*self).update(p);
            }
        }
    };
}

stringify_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
stringify_float!(f32, f64);

impl_stringify!(&'static str, bool, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64);
