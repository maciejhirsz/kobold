use crate::{Html, Mountable};
use crate::util;
use wasm_bindgen::JsValue;
use web_sys::Node;

/// Helper trait for wrapping iterators in [`List`](List)s which implement [`Html`](Html).
pub trait ListExt: Sized {
    fn list(self) -> List<Self>;
}

impl<T> ListExt for T
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Html,
{
    fn list(self) -> List<Self> {
        List(self)
    }
}

/// Wrapper type that implements `Html` for iterators.
pub struct List<T>(T);

pub struct ListProduct<T> {
    list: Vec<T>,
    visible: usize,
    node: Node,
}

impl<T: 'static> Mountable for ListProduct<T> {
    fn js(&self) -> &JsValue {
        &self.node
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
        let node = util::__kobold_create_div();

        let list: Vec<_> = iter
            .map(|item| {
                let built = item.build();

                built.mount(&node);

                built
            })
            .collect();

        let visible = list.len();

        ListProduct {
            list,
            visible,
            node,
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
                old.unmount(&p.node);
            }
            p.visible = updated;
        } else {
            for (old, new) in p.list[updated..].iter_mut().zip(&mut new) {
                new.update(old);
                old.mount(&p.node);

                p.visible += 1;
            }

            for new in new {
                let built = new.build();

                built.mount(&p.node);

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
