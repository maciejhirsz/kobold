// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use std::mem::MaybeUninit;
use std::pin::Pin;

use web_sys::Node;

use crate::dom::{Anchor, Fragment, FragmentBuilder};
use crate::internal::{In, Out};
use crate::{Mountable, View};

mod linked_list;

/// Wrapper type that implements `View` for iterators, created by the
/// [`for`](crate::keywords::for) keyword.
#[repr(transparent)]
pub struct List<T>(pub(crate) T);

pub struct ListProduct<T> {
    list: Vec<Pin<Box<MaybeUninit<T>>>>,
    visible: usize,
    fragment: FragmentBuilder,
}

impl<T> Anchor for ListProduct<T> {
    type Js = Node;
    type Target = Fragment;

    fn anchor(&self) -> &Fragment {
        &self.fragment
    }
}

fn uninit_box<T>() -> Pin<Box<MaybeUninit<T>>> {
    use std::alloc::{alloc, Layout};

    unsafe {
        Pin::new_unchecked(Box::from_raw(
            alloc(Layout::new::<MaybeUninit<T>>()) as *mut MaybeUninit<T>
        ))
    }
}

impl<T> View for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = ListProduct<<T::Item as View>::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        let iter = self.0.into_iter();
        let fragment = FragmentBuilder::new();

        let list: Vec<_> = iter
            .map(|item| {
                let mut b = uninit_box();
                let built = In::pinned(b.as_mut(), |p| item.build(p));

                fragment.append(built.js());

                b
            })
            .collect();

        p.put(ListProduct {
            visible: list.len(),
            list,
            fragment,
        })
    }

    fn update(self, p: &mut Self::Product) {
        let mut new = self.0.into_iter();
        let mut updated = 0;

        for (old, new) in p.list[..p.visible].iter_mut().zip(&mut new) {
            new.update(unsafe { old.as_mut().get_unchecked_mut().assume_init_mut() });
            updated += 1;
        }

        if p.visible > updated {
            for old in p.list[updated..p.visible].iter_mut() {
                unsafe {
                    old.as_ref().assume_init_ref().unmount();
                    old.as_mut().get_unchecked_mut().assume_init_drop();
                }
            }
            p.list.truncate(10);
            p.visible = updated;
        } else {
            for (old, new) in p.list[updated..].iter_mut().zip(&mut new) {
                let built = In::pinned(old.as_mut(), |p| new.build(p));

                p.fragment.append(built.js());
                p.visible += 1;
            }

            p.list.reserve(new.size_hint().0);

            for new in new {
                let mut b = uninit_box();
                let built = In::pinned(b.as_mut(), |p| new.build(p));

                p.fragment.append(built.js());
                p.list.push(b);
                p.visible += 1;
            }
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
