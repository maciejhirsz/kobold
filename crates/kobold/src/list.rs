// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for rendering lists

use web_sys::Node;

use crate::dom::{Fragment, FragmentBuilder};
use crate::{Mountable, View};

mod log;

pub use log::Log;

/// Wrapper type that implements `View` for iterators. Use the [`list`](ListIteratorExt::list)
/// method on the iterator to create one.
#[repr(transparent)]
pub struct List<T>(pub(crate) T);

pub struct ListProduct<T> {
    list: Vec<T>,
    visible: usize,
    fragment: FragmentBuilder,
}

impl<T: 'static> Mountable for ListProduct<T> {
    type Js = Node;
    type Anchor = Fragment;

    fn anchor(&self) -> &Fragment {
        &self.fragment
    }
}

#[doc(hidden)]
pub trait ListIteratorExt: Iterator + Sized {
    #[doc(hidden)]
    #[deprecated(since = "0.6.0", note = "please use `{ for <expression> }` instead")]
    fn list(self) -> List<Self> {
        List(self)
    }
}

#[doc(hidden)]
impl<T: Iterator> ListIteratorExt for T {}

impl<T> View for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = ListProduct<<T::Item as View>::Product>;

    fn build(self) -> Self::Product {
        let iter = self.0.into_iter();
        let fragment = FragmentBuilder::new();

        let list: Vec<_> = iter
            .map(|item| {
                let built = item.build();

                fragment.append(built.js());

                built
            })
            .collect();

        let visible = list.len();

        ListProduct {
            list,
            visible,
            fragment,
        }
    }

    fn update(self, p: &mut Self::Product) {
        let mut new = self.0.into_iter();
        let mut updated = 0;

        for (old, new) in p.list[..p.visible].iter_mut().zip(&mut new) {
            new.update(old);
            updated += 1;
        }

        if p.visible > updated {
            for old in p.list[updated..p.visible].iter() {
                old.unmount();
            }
            p.visible = updated;
        } else {
            for (old, new) in p.list[updated..].iter_mut().zip(&mut new) {
                new.update(old);

                p.fragment.append(old.js());
                p.visible += 1;
            }

            for new in new {
                let built = new.build();

                p.fragment.append(built.js());
                p.list.push(built);
                p.visible += 1;
            }
        }
    }
}

impl<H: View> View for Vec<H> {
    type Product = ListProduct<H::Product>;

    fn build(self) -> Self::Product {
        List(self).build()
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p);
    }
}

impl<'a, H> View for &'a [H]
where
    &'a H: View,
{
    type Product = ListProduct<<&'a H as View>::Product>;

    fn build(self) -> Self::Product {
        List(self).build()
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p)
    }
}

impl<H: View, const N: usize> View for [H; N] {
    type Product = ListProduct<H::Product>;

    fn build(self) -> Self::Product {
        List(self).build()
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p)
    }
}
