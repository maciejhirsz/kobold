//! Utilities for rendering lists

use crate::{Element, Html, Mountable};

/// Wrapper type that implements `Html` for iterators. Use the [`list`](ListIteratorExt::list)
/// method on the iterator to create one.
#[repr(transparent)]
pub struct List<T>(T);

pub struct ListProduct<T> {
    list: Vec<T>,
    visible: usize,
    fragment: Element,
}

impl<T: 'static> Mountable for ListProduct<T> {
    fn el(&self) -> &Element {
        &self.fragment
    }
}

pub trait ListIteratorExt: Sized {
    fn list(self) -> List<Self>;
}

impl<T> ListIteratorExt for T
where
    T: Iterator,
    <T as Iterator>::Item: Html,
{
    fn list(self) -> List<Self> {
        List(self)
    }
}

impl<T> Html for List<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Html,
{
    type Product = ListProduct<<T::Item as Html>::Product>;

    fn build(self) -> Self::Product {
        let iter = self.0.into_iter();
        let fragment = Element::new_fragment();

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

impl<H: Html> Html for Vec<H> {
    type Product = ListProduct<H::Product>;

    fn build(self) -> Self::Product {
        List(self).build()
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p);
    }
}

impl<H: Html, const N: usize> Html for [H; N] {
    type Product = ListProduct<H::Product>;

    fn build(self) -> Self::Product {
        List(self).build()
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p)
    }
}

impl<'a, H> Html for &'a [H]
where
    &'a H: Html,
{
    type Product = ListProduct<<&'a H as Html>::Product>;

    fn build(self) -> Self::Product {
        List(self).build()
    }

    fn update(self, p: &mut Self::Product) {
        List(self).update(p)
    }
}
