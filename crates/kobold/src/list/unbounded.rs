// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::Range;
use std::slice::Iter;

use crate::dom::FragmentBuilder;
use crate::internal::In;
use crate::list::Collection;
use crate::{Mountable, View};

impl<P> Collection<P> for Vec<Box<P>> {
    type Iter<'a> = ProductIter<'a, P> where P: 'a;

    fn new() -> Self {
        Vec::new()
    }

    fn extend_list<I>(&mut self, iter: I, frag: &FragmentBuilder)
    where
        P: Mountable,
        I: Iterator,
        I::Item: View<Product = P>,
    {
        self.extend(iter.map(|view| {
            let built = In::boxed(|p| view.build(p));

            frag.append(built.js());

            built
        }));
    }

    fn products(&self, range: Range<usize>) -> Self::Iter<'_> {
        ProductIter {
            inner: self[range].iter(),
        }
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }
}

pub struct ProductIter<'a, P> {
    inner: Iter<'a, Box<P>>,
}

impl<'a, P> Iterator for ProductIter<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|i| &**i)
    }
}
