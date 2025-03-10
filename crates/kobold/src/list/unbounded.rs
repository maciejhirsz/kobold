// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use web_sys::Node;

use crate::dom::{Anchor, Fragment, FragmentBuilder};
use crate::internal::{In, Out};
use crate::{Mountable, View};

pub struct ListProduct<P: Mountable> {
    list: Vec<Box<P>>,
    mounted: usize,
    fragment: FragmentBuilder,
}

impl<P: Mountable> ListProduct<P> {
    pub fn build<I>(iter: I, p: In<Self>) -> Out<Self>
    where
        I: Iterator,
        I::Item: View<Product = P>,
    {
        let mut list = p.put(ListProduct {
            list: Vec::new(),
            mounted: 0,
            fragment: FragmentBuilder::new(),
        });

        list.extend(iter);
        list
    }

    pub fn update<I>(&mut self, mut iter: I)
    where
        I: Iterator,
        I::Item: View<Product = P>,
    {
        let mut updated = 0;

        while let Some(old) = self.list.get_mut(updated) {
            let Some(new) = iter.next() else {
                break;
            };

            new.update(old);
            updated += 1;
        }

        if updated < self.mounted {
            self.unmount(updated);
        } else {
            self.mount(updated);

            if updated == self.list.len() {
                self.extend(iter);
            }
        }
    }

    fn extend<I>(&mut self, iter: I)
    where
        I: Iterator,
        I::Item: View<Product = P>,
    {
        self.list.extend(iter.map(|view| {
            let built = In::boxed(|p| view.build(p));

            self.fragment.append(built.js());

            built
        }));

        self.mounted = self.list.len();
    }

    fn unmount(&mut self, from: usize) {
        debug_assert!(self.list.get(from..self.mounted).is_some());

        for p in unsafe { self.list.get_unchecked(from..self.mounted).iter() } {
            p.unmount();
        }
        self.mounted = from;
    }

    fn mount(&mut self, to: usize) {
        debug_assert!(self.list.get(self.mounted..to).is_some());

        for p in unsafe { self.list.get_unchecked(self.mounted..to).iter() } {
            self.fragment.append(p.js());
        }
        self.mounted = to;
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
