//! Utilities for rendering lists

use web_sys::Node;

use crate::dom::Fragment;
use crate::{Element, View, Mountable};

/// Wrapper type that implements `View` for iterators. Use the [`list`](ListIteratorExt::list)
/// method on the iterator to create one.
#[repr(transparent)]
pub struct List<T>(T);

pub struct ListProduct<T> {
    list: Vec<T>,
    visible: usize,
    fragment: Fragment,
}

impl<T: 'static> Mountable for ListProduct<T> {
    type Js = Node;

    fn el(&self) -> &Element {
        &self.fragment
    }
}

pub trait ListIteratorExt: Iterator + Sized {
    fn list(self) -> List<Self> {
        List(self)
    }
}

impl<T: Iterator> ListIteratorExt for T {}

impl<T> View for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: View,
{
    type Product = ListProduct<<T::Item as View>::Product>;

    fn build(self) -> Self::Product {
        let iter = self.0.into_iter();
        let fragment = Fragment::new();

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
                old.el().unmount();
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
