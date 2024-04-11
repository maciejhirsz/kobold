// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use crate::internal::{In, Out};
use crate::View;

pub mod unbounded;

use unbounded::ListProduct;

/// Wrapper type that implements `View` for iterators, created by the
/// [`for`](crate::keywords::for) keyword.
#[repr(transparent)]
pub struct List<T>(pub(crate) T);

impl<T> View for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = ListProduct<<T::Item as View>::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        ListProduct::build(self.0.into_iter(), p)
    }

    fn update(self, p: &mut Self::Product) {
        p.update(self.0.into_iter());
    }
}


impl<V: View> View for Vec<V> {
    type Product = ListProduct<V::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        List(self).build(p)
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p);
    }
}

impl<'a, V> View for &'a [V]
where
    &'a V: View,
{
    type Product = ListProduct<<&'a V as View>::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        List(self).build(p)
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p)
    }
}

impl<V: View, const N: usize> View for [V; N] {
    type Product = ListProduct<V::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        List(self).build(p)
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p)
    }
}
