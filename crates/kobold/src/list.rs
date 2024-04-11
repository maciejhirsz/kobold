// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use std::marker::PhantomData;

use crate::internal::{In, Out};
use crate::View;

pub mod bounded;
pub mod unbounded;

use bounded::BoundedProduct;
use unbounded::ListProduct;

/// Zero-sized marker making the [`List`] unbounded: it can grow to arbitrary
/// size but will require memory allocation.
pub struct Unbounded;

/// Zero-sized marker making the [`List`] bounded to a max length of `N`:
/// elements over the limit are ignored and no allocations are made.
pub struct Bounded<const N: usize>;

/// Wrapper type that implements `View` for iterators, created by the
/// [`for`](crate::keywords::for) keyword.
#[repr(transparent)]
pub struct List<T, B = Unbounded>(T, PhantomData<B>);

impl<T> List<T> {
    pub const fn new(item: T) -> Self {
        List(item, PhantomData)
    }

    pub const fn new_bounded<const N: usize>(item: T) -> List<T, Bounded<N>> {
        List(item, PhantomData)
    }
}

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

impl<T, const N: usize> View for List<T, Bounded<N>>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = BoundedProduct<<T::Item as View>::Product, N>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        BoundedProduct::build(self.0.into_iter(), p)
    }

    fn update(self, p: &mut Self::Product) {
        p.update(self.0.into_iter());
    }
}

impl<V: View> View for Vec<V> {
    type Product = ListProduct<V::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        List::new(self).build(p)
    }

    fn update(self, p: &mut Self::Product) {
        List::new(self).update(p);
    }
}

impl<'a, V> View for &'a [V]
where
    &'a V: View,
{
    type Product = ListProduct<<&'a V as View>::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        List::new(self).build(p)
    }

    fn update(self, p: &mut Self::Product) {
        List::new(self).update(p)
    }
}

impl<V: View, const N: usize> View for [V; N] {
    type Product = ListProduct<V::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        List::new(self).build(p)
    }

    fn update(self, p: &mut Self::Product) {
        List::new(self).update(p)
    }
}
