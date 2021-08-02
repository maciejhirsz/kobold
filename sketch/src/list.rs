use crate::traits::{Html, Mountable, Update};
use crate::util;
use web_sys::Node;

pub struct IterWrapper<T>(pub T);

pub struct RenderedList<T> {
    list: Vec<T>,
    node: Node,
}

impl<T> Mountable for RenderedList<T> {
    fn node(&self) -> &Node {
        &self.node
    }
}

impl<T> Update<T> for <Vec<T::Item> as Html>::Rendered
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Html,
{
    #[inline]
    fn update(&mut self, new: T) {
        self.update(IterWrapper(new))
    }
}

impl<T> Update<IterWrapper<T>> for RenderedList<<<T as IntoIterator>::Item as Html>::Rendered>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Html,
{
    fn update(&mut self, new: IterWrapper<T>) {
        let mut new = new.0.into_iter();
        let mut updated = 0;

        for (old, new) in self.list.iter_mut().zip(&mut new) {
            old.update(new);
            updated += 1;
        }

        if self.list.len() > updated {
            for old in self.list.drain(updated..) {
                old.unmount(&self.node);
            }
        } else {
            for new in new {
                let rendered = new.render();

                rendered.mount(&self.node);

                self.list.push(rendered);
            }
        }
    }
}

impl<T> Html for IterWrapper<T>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Html,
{
    type Rendered = RenderedList<<T::Item as Html>::Rendered>;

    fn render(self) -> Self::Rendered {
        let iter = self.0.into_iter();
        let mut list: Vec<<<T as IntoIterator>::Item as Html>::Rendered> = Vec::with_capacity(iter.size_hint().0);

        let node = util::__sketch_create_el("div");

        for item in iter {
            let rendered = item.render();

            rendered.mount(&node);

            list.push(rendered);
        }

        RenderedList {
            list,
            node,
        }
    }
}

impl<H: Html> Html for Vec<H> {
    type Rendered = RenderedList<H::Rendered>;

    fn render(self) -> Self::Rendered {
        let mut list: Vec<H::Rendered> = Vec::with_capacity(self.len());

        let node = util::__sketch_create_el("div");

        for item in self {
            let rendered = item.render();

            rendered.mount(&node);

            list.push(rendered);
        }

        RenderedList {
            list,
            node,
        }
    }
}

impl<H: Html, const N: usize> Html for [H; N] {
    type Rendered = RenderedList<H::Rendered>;

    fn render(self) -> Self::Rendered {
        let mut list: Vec<H::Rendered> = Vec::with_capacity(self.len());

        let node = util::__sketch_create_el("div");

        for item in self {
            let rendered = item.render();

            rendered.mount(&node);

            list.push(rendered);
        }

        RenderedList {
            list,
            node,
        }
    }
}
