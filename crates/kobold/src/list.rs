// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use web_sys::Node;

use crate::dom::{Anchor, Fragment, FragmentBuilder};
use crate::internal::{In, Out};
use crate::{Mountable, View};

mod linked_list;
mod linked_list_dyn;
mod page_list;

use page_list::LinkedList;

/// Wrapper type that implements `View` for iterators, created by the
/// [`for`](crate::keywords::for) keyword.
#[repr(transparent)]
pub struct List<T>(pub(crate) T);

pub struct ListProduct<P: Mountable> {
    list: LinkedList<AutoUnmount<P>>,
    fragment: FragmentBuilder,
}

struct AutoUnmount<P: Mountable>(P);

impl<P> Drop for AutoUnmount<P>
where
    P: Mountable,
{
    fn drop(&mut self) {
        self.0.unmount();
    }
}

impl<P> Anchor for ListProduct<P>
where
    P: Mountable,
{
    type Js = Node;
    type Target = Fragment;

    fn anchor(&self) -> &Fragment {
        &self.fragment
    }
}

impl<T> View for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = ListProduct<<T::Item as View>::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        let fragment = FragmentBuilder::new();

        let list = LinkedList::build(self.0, |view, b| {
            let built = view.build(unsafe { b.cast() });

            fragment.append(built.js());

            unsafe { built.cast() }
        });

        p.put(ListProduct { list, fragment })
    }

    fn update(self, p: &mut Self::Product) {
        let mut new = self.0.into_iter();
        let mut old = p.list.cursor();

        old.pair(&mut new, |old, new| new.update(&mut old.0));

        old.truncate_rest().extend(new, |view, b| {
            let built = view.build(unsafe { b.cast() });

            p.fragment.append(built.js());

            unsafe { built.cast() }
        });
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
