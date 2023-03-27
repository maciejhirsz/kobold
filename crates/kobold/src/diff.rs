use std::ops::Deref;

use web_sys::Node;

use crate::attribute::AttributeView;
use crate::dom::Element;
use crate::value::IntoText;
use crate::View;

pub trait Diff: Copy {
    type Memo: 'static;

    fn into_memo(self) -> Self::Memo;

    fn diff(self, memo: &mut Self::Memo) -> bool;

    #[doc(hidden)]
    #[deprecated(since = "0.6.0", note = "please use `{ static <expression> }` instead")]
    fn no_diff(self) -> NoDiff<Self> {
        NoDiff(self)
    }
}

macro_rules! impl_diff_str {
    ($($ty:ty),*) => {
        $(
            impl Diff for $ty {
                type Memo = String;

                fn into_memo(self) -> String {
                    self.into()
                }

                fn diff(self, memo: &mut String) -> bool {
                    if self != memo {
                        self.clone_into(memo);
                        true
                    } else {
                        false
                    }
                }
            }
        )*
    };
}

macro_rules! impl_diff {
    ($($ty:ty),*) => {
        $(
            impl Diff for $ty {
                type Memo = $ty;

                fn into_memo(self) -> $ty {
                    self
                }

                fn diff(self, memo: &mut $ty) -> bool {
                    if self != *memo {
                        *memo = self;
                        true
                    } else {
                        false
                    }
                }
            }
        )*
    };
}

impl_diff_str!(&str, &String);
impl_diff!(bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

#[repr(transparent)]
pub struct RefDiff<'a, T: ?Sized>(pub(crate) &'a T);

impl<T: ?Sized> Clone for RefDiff<'_, T> {
    fn clone(&self) -> Self {
        RefDiff(self.0)
    }
}

impl<T: ?Sized> Copy for RefDiff<'_, T> {}

impl<T: ?Sized> Deref for RefDiff<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized> AsRef<T> for RefDiff<'_, T> {
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized> Diff for RefDiff<'_, T> {
    type Memo = *const ();

    fn into_memo(self) -> Self::Memo {
        self.0 as *const _ as *const ()
    }

    fn diff(self, memo: &mut Self::Memo) -> bool {
        let ptr = self.0 as *const _ as *const ();

        if ptr != *memo {
            *memo = ptr;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NoDiff<T, const U: bool = false>(pub(crate) T);

impl<T, const U: bool> Deref for NoDiff<T, U> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T, const U: bool> View for NoDiff<T, U>
where
    T: IntoText + Copy,
{
    type Product = Element;

    fn build(self) -> Self::Product {
        Element::new(self.into_text())
    }

    fn update(self, _: &mut Self::Product) {}
}

impl<T, P, const U: bool> AttributeView<P> for NoDiff<T, U>
where
    T: AttributeView<P>,
{
    type Product = ();

    fn build(self) {}

    fn build_in(self, prop: P, node: &Node) {
        self.0.build_in(prop, node);
    }

    fn update_in(self, _: P, _: &Node, _: &mut ()) {}
}

impl<T, const U: bool> Diff for NoDiff<T, U>
where
    T: Copy,
{
    type Memo = ();

    fn into_memo(self) {}

    fn diff(self, _: &mut ()) -> bool {
        U
    }
}

impl<const U: bool> AsRef<str> for NoDiff<&str, U> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[doc(hidden)]
pub trait StrExt {
    #[deprecated(since = "0.6.0", note = "please use `{ ref <expression> }` instead")]
    fn fast_diff(&self) -> RefDiff<str>;
}

#[doc(hidden)]
impl StrExt for str {
    fn fast_diff(&self) -> RefDiff<str> {
        RefDiff(self)
    }
}

// /// A borrowed string that's diffed by pointer instead of value.
// #[repr(transparent)]
// #[derive(Clone, Copy)]
// pub struct FastDiff<'a>(&'a str);

// impl Deref for FastDiff<'_> {
//     type Target = str;

//     fn deref(&self) -> &Self::Target {
//         self.0
//     }
// }

// impl AsRef<str> for FastDiff<'_> {
//     fn as_ref(&self) -> &str {
//         self.0
//     }
// }
