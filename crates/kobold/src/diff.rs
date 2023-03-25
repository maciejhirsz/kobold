use std::ops::Deref;

use crate::dom::Element;
use crate::value::IntoText;
use crate::View;

pub trait Diff: Copy {
    type Memo: 'static;

    fn into_memo(self) -> Self::Memo;

    fn diff(self, memo: &mut Self::Memo) -> bool;

    #[inline]
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

impl Diff for FastDiff<'_> {
    type Memo = usize;

    fn into_memo(self) -> usize {
        self.as_ptr() as _
    }

    fn diff(self, state: &mut usize) -> bool {
        if self.as_ptr() as usize != *state {
            *state = self.as_ptr() as _;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NoDiff<T>(T);

impl<T> Deref for NoDiff<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: IntoText + Copy> View for NoDiff<T> {
    type Product = Element;

    fn build(self) -> Self::Product {
        Element::new(self.into_text())
    }

    fn update(self, _: &mut Self::Product) {}
}

impl<T> Diff for NoDiff<T>
where
    T: Copy,
{
    type Memo = ();

    fn into_memo(self) {}

    fn diff(self, _: &mut ()) -> bool {
        false
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
#[derive(Clone, Copy)]
pub struct FastDiff<'a>(&'a str);

impl Deref for FastDiff<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
