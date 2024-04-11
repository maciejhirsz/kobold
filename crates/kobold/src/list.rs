// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use std::ops::Range;

use web_sys::Node;

use crate::dom::{Anchor, Fragment, FragmentBuilder};
use crate::internal::{In, Out};
use crate::{Mountable, View};

/// Wrapper type that implements `View` for iterators, created by the
/// [`for`](crate::keywords::for) keyword.
#[repr(transparent)]
pub struct List<T>(pub(crate) T);

pub struct ListProduct<P: Mountable> {
    list: Vec<Box<P>>,
    mounted: usize,
    fragment: FragmentBuilder,
}

impl<P: Mountable> ListProduct<P> {
    fn extend<I>(&mut self, iter: I)
    where
        I: Iterator,
        I::Item: View<Product = P>
    {
        self.list.extend(iter.map(|view| {
            let built = In::boxed(|p| view.build(p));

            self.fragment.append(built.js());

            built
        }));

        self.mounted = self.list.len();
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

trait Collection<P> {
    type Iter<'a>: Iterator<Item = &'a P>
    where
        Self: 'a,
        P: 'a;

    fn new() -> Self;

    fn extend_list<I>(&mut self, iter: I, frag: &FragmentBuilder)
    where
        P: Mountable,
        I: Iterator,
        I::Item: View<Product = P>;

    fn products(&self, range: Range<usize>) -> Self::Iter<'_>;

    fn len(&self) -> usize;
}

impl<T> View for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = ListProduct<<T::Item as View>::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        let mut list = p.put(ListProduct {
            list: Vec::new(),
            mounted: 0,
            fragment: FragmentBuilder::new(),
        });

        list.extend(self.0.into_iter());
        list
    }

    fn update(self, p: &mut Self::Product) {
        // `mounted` is always within the bounds of `len`, this
        // convinces the compiler that this is indeed the fact,
        // so it can optimize bounds checks here.
        if p.mounted > p.list.len() {
            unsafe { std::hint::unreachable_unchecked() }
        }

        let mut new = self.0.into_iter();
        let mut consumed = 0;

        while let Some(old) = p.list.get_mut(consumed) {
            let Some(new) = new.next() else {
                break;
            };

            new.update(old);
            consumed += 1;
        }

        if consumed < p.mounted {
            for tail in p.list[consumed..p.mounted].iter() {
                tail.unmount();
            }
            p.mounted = consumed;
        } else {
            for built in p.list[p.mounted..consumed].iter() {
                p.fragment.append(built.js());
            }

            p.extend(new);
        }
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
