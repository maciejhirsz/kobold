// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use web_sys::Node;

use crate::diff::{Diff, RefDiff};
use crate::dom::{LargeInt, Property, TextContent};
use crate::util;

use crate::{Element, Mountable, View};

/// Value that can be set as a property on DOM node
pub trait Value<P> {
    fn set_prop(self, prop: P, node: &Node);
}

/// Value that can be turned into a DOM `Text` node
pub trait IntoText {
    fn into_text(self) -> Node;
}

macro_rules! impl_text {
    ($($util:ident [$($ty:ty),*])*) => {
        $(
            $(
                impl IntoText for $ty {
                    fn into_text(self) -> Node {
                        util::$util(self as _)
                    }
                }
            )*
        )*
    };
}

impl_text! {
    text_node [&str, &String]
    text_node_num [i8, i16, i32, isize, u8, u16, u32, usize, f32, f64]
    text_node_bool [bool]
}

macro_rules! impl_value {
    ($abi:ty: $($ty:ty),*) => {
        $(
            impl<P> Value<P> for $ty
            where
                P: for<'a> Property<$abi>,
            {
                fn set_prop(self, prop: P, node: &Node) {
                    prop.set(node, self as _);
                }
            }
        )*
    };
}

impl_value!(&'a str: &str, &String);
impl_value!(bool: bool);
impl_value!(f64: u8, u16, u32, usize, i8, i16, i32, isize, f32, f64);

impl<P> Value<P> for RefDiff<'_, str>
where
    P: for<'a> Property<&'a str>,
{
    fn set_prop(self, prop: P, node: &Node) {
        prop.set(node, &self);
    }
}

pub struct TextProduct<M> {
    memo: M,
    el: Element,
}

impl<M: 'static> Mountable for TextProduct<M> {
    type Js = web_sys::Text;

    fn el(&self) -> &Element {
        &self.el
    }
}

impl View for String {
    type Product = TextProduct<String>;

    fn build(self) -> Self::Product {
        let el = Element::new_text(self.as_str());

        TextProduct { memo: self, el }
    }

    fn update(self, p: &mut Self::Product) {
        if p.memo != self {
            p.memo = self;
            p.memo.set_prop(TextContent, &p.el);
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

            impl<P> Value<P> for $t
            where
                P: Property<f64> + for<'a> Property<&'a str>,
            {
                fn set_prop(self, prop: P, el: &Node) {
                    match <$d>::try_from(self) {
                        Ok(int) => prop.set(el, int as f64),
                        Err(_) => self.stringify(|s| prop.set(el, s)),
                    }
                }
            }

            impl IntoText for $t {
                fn into_text(self) -> Node {
                    match <$d>::try_from(self) {
                        Ok(downcast) => downcast.into_text(),
                        Err(_) => self.stringify(util::text_node),
                    }
                }
            }
        )*
    };
}

large_int!(u64 > u32, u128 > u32, i64 > i32, i128 > i32);

macro_rules! impl_text_view {
    ($($ty:ty),*) => {
        $(
            impl View for $ty {
                type Product = TextProduct<<Self as Diff>::Memo>;

                fn build(self) -> Self::Product {
                    let el = Element::new(self.into_text());
                    let memo = self.into_memo();

                    TextProduct { memo, el }
                }

                fn update(self, p: &mut Self::Product) {
                    if self.diff(&mut p.memo) {
                        self.set_prop(TextContent, &p.el);
                    }
                }
            }
        )*
    };
}

impl_text_view!(&str, &String, RefDiff<'_, str>);
impl_text_view!(bool, u8, u16, u32, u64, u128, usize, isize, i8, i16, i32, i64, i128, f32, f64);

impl<'a> View for &&'a str {
    type Product = <&'a str as View>::Product;

    fn build(self) -> Self::Product {
        (*self).build()
    }

    fn update(self, p: &mut Self::Product) {
        (*self).update(p)
    }
}

macro_rules! impl_ref_view {
    ($($ty:ty),*) => {
        $(
            impl View for &$ty {
                type Product = <$ty as View>::Product;

                fn build(self) -> Self::Product {
                    (*self).build()
                }

                fn update(self, p: &mut Self::Product) {
                    (*self).update(p)
                }
            }
        )*
    };
}

impl_ref_view!(bool, u8, u16, u32, u64, u128, usize, isize, i8, i16, i32, i64, i128, f32, f64);
