// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

use web_sys::Node;

use crate::dom::{Anchor, Fragment, FragmentBuilder};
use crate::runtime::{EventContext, Then, Trigger, UsedEvents};
use crate::{Mountable, View};

pub struct BoundedProduct<P: Mountable, const N: usize> {
    list: BoundedVec<P, N>,
    mounted: usize,
    fragment: FragmentBuilder,
}

impl<P: Mountable, const N: usize> BoundedProduct<P, N> {
    pub fn build<I>(iter: I) -> Self
    where
        I: Iterator,
        I::Item: View<Product = P>,
    {
        let mut list = BoundedProduct {
            list: BoundedVec::new(),
            mounted: 0,
            fragment: FragmentBuilder::new(),
        };

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
            let built = view.build();

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

impl<P, const N: usize> Anchor for BoundedProduct<P, N>
where
    P: Mountable,
{
    const EVENTS: UsedEvents = P::EVENTS;

    type Js = Node;
    type Target = Fragment;

    fn anchor(&self) -> &Fragment {
        &self.fragment
    }
}

impl<P, const N: usize> Trigger for BoundedProduct<P, N>
where
    P: Mountable,
{
    fn trigger<C: EventContext>(&mut self, ctx: &mut C) -> Option<Then> {
        debug_assert!(self.list.get(..self.mounted).is_some());

        let list = unsafe { self.list.get_unchecked_mut(..self.mounted) };

        list.iter_mut().find_map(|p| p.trigger(ctx))
    }
}

pub struct BoundedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> BoundedVec<T, N> {
    pub fn push(&mut self, item: T) -> bool {
        if self.len >= N {
            return false;
        }

        self.data[self.len].write(item);
        self.len += 1;

        true
    }
}

impl<T, const N: usize> BoundedVec<T, N> {
    const fn new() -> Self {
        BoundedVec {
            data: [const { MaybeUninit::<T>::uninit() }; N],
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn extend<I>(&mut self, mut iter: I)
    where
        I: Iterator<Item = T>,
    {
        while self.len < N {
            let Some(item) = iter.next() else {
                break;
            };

            self.push(item);
        }
    }
}

impl<T, const N: usize> Deref for BoundedVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        debug_assert!(self.len <= N);

        unsafe { &*(self.data.get_unchecked(..self.len) as *const [_] as *const [T]) }
    }
}

impl<T, const N: usize> DerefMut for BoundedVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(self.len <= N);

        unsafe { &mut *(self.data.get_unchecked_mut(..self.len) as *mut [_] as *mut [T]) }
    }
}

impl<T, const N: usize> Drop for BoundedVec<T, N> {
    fn drop(&mut self) {
        unsafe { std::ptr::drop_in_place(&mut **self as *mut [T]) }
    }
}
