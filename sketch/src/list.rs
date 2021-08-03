use crate::traits::{Html, Mountable, Update};
use crate::util;
use web_sys::Node;

pub struct IterWrapper<T>(pub T);

pub struct RenderedList<T> {
    list: Vec<T>,
    visible: usize,
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

        for (old, new) in self.list[..self.visible].iter_mut().zip(&mut new) {
            old.update(new);
            updated += 1;
        }

        if self.visible > updated {
            for old in self.list[updated..].iter() {
                old.unmount(&self.node);
            }
            self.visible = updated;
        } else {
            for (old, new) in self.list[updated..].iter_mut().zip(&mut new) {
                old.update(new);
                old.mount(&self.node);

                self.visible += 1;
            }

            for new in new {
                let rendered = new.render();

                rendered.mount(&self.node);

                self.list.push(rendered);
                self.visible += 1;
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
        let mut list: Vec<<<T as IntoIterator>::Item as Html>::Rendered> =
            Vec::with_capacity(iter.size_hint().0);

        let node = util::__sketch_create_div();

        for item in iter {
            let rendered = item.render();

            rendered.mount(&node);

            list.push(rendered);
        }

        let visible = list.len();

        RenderedList {
            list,
            visible,
            node,
        }
    }
}

impl<H: Html> Html for Vec<H> {
    type Rendered = RenderedList<H::Rendered>;

    #[inline]
    fn render(self) -> Self::Rendered {
        IterWrapper(self).render()
    }
}

impl<H: Html, const N: usize> Html for [H; N] {
    type Rendered = RenderedList<H::Rendered>;

    #[inline]
    fn render(self) -> Self::Rendered {
        IterWrapper(self).render()
    }
}
