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

impl Html for &'static str {
    type Product = ValueProduct<&'static str>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self);

        ValueProduct { value: self, el }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self {
            p.value = self;
            p.el.set_text(self);
        }
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

fn bool_to_str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

impl Html for bool {
    type Product = ValueProduct<bool>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(bool_to_str(self));

        ValueProduct { value: self, el }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self {
            p.value = self;
            p.el.set_text(bool_to_str(p.value));
        }
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    let mut buf = itoa::Buffer::new();

                    let el = Element::new_text(buf.format(self));

                    ValueProduct { value: self, el }
                }

                fn update(self, p: &mut Self::Product) {
                    if p.value != self {
                        p.value = self;

                        let mut buf = itoa::Buffer::new();

                        p.el.set_text(buf.format(self));
                    }
                }
            }

            impl_ref_copy!($t);
        )*
    };
}

macro_rules! impl_float {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    let mut buf = ryu::Buffer::new();

                    let el = Element::new_text(buf.format(self));

                    ValueProduct { value: self, el }
                }

                fn update(self, p: &mut Self::Product) {
                    if (p.value - self).abs() > <$t>::EPSILON {
                        p.value = self;

                        let mut buf = ryu::Buffer::new();

                        p.el.set_text(buf.format(self));
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

impl_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
impl_float!(f32, f64);
impl_ref_copy!(bool);
impl_ref_copy!(&'static str);
