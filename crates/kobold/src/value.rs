use crate::{Element, Html, Mountable};
use crate::prelude::{ShouldRender, IntoState};
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

pub trait Stringify {
    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R;
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

#[derive(PartialEq, Eq)]
pub struct StringHash {
    hash: u64,
}

impl From<&str> for StringHash {
    fn from(s: &str) -> StringHash {

        let hash = if s.len() > 32 {
            (s.len() as u64) | ((s.as_ptr() as u64) << 32)
        } else {
            use std::hash::{Hash, Hasher};

            let mut hasher = fnv::FnvHasher::default();

            s.hash(&mut hasher);

            hasher.finish()
        };

        StringHash {
            hash,
        }
    }
}

impl Html for &str {
    type Product = ValueProduct<StringHash>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self);

        ValueProduct { value: self.into(), el }
    }

    fn update(self, p: &mut Self::Product) {
        let new = self.into();

        if p.value != new {
            p.value = new;
            p.el.set_text(self);
        }
    }
}

impl Html for &&str {
    type Product = ValueProduct<StringHash>;

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
            state.replace_range(.., self);
            ShouldRender::Yes
        } else {
            ShouldRender::No
        }
    }
}

stringify_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
stringify_float!(f32, f64);

impl_stringify!(
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    usize,
    isize,
    f32,
    f64
);
