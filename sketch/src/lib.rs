// use std::any::Any;
use beef::Cow;
use web_sys::Document;

pub use web_sys::Node;
pub use sketch_macro::html;

pub mod reexport {
    pub use web_sys;
    pub use wasm_bindgen;
}

pub trait Html: Sized {
    type Rendered: Update<Self> + Mountable;

    fn render(self) -> Self::Rendered;
}

pub trait Update<H> {
    fn update(&mut self, new: H);
}

pub trait Mountable {
    fn node(&self) -> &Node;

    fn mount(&self, parent: &Node) {
        parent.append_child(&self.node()).unwrap();
    }

    fn unmount(&self, parent: &Node) {
        parent.remove_child(&self.node()).unwrap();
    }
}

impl Html for () {
    type Rendered = EmptyNode;

    fn render(self) -> EmptyNode {
        EmptyNode(document().create_text_node("").into())
    }
}

pub struct EmptyNode(Node);

impl Mountable for EmptyNode {
    fn node(&self) -> &Node {
        &self.0
    }
}

impl Update<()> for EmptyNode {
    fn update(&mut self, _: ()) {}
}

pub struct RenderedText {
    text: Cow<'static, str>,
    node: Node,
}

impl Mountable for RenderedText {
    fn node(&self) -> &Node {
        &self.node
    }
}

pub fn document() -> Document {
    let window = web_sys::window().expect("no window exists");
    window.document().expect("window should have a document")
}

impl Html for &'static str {
    type Rendered = RenderedText;

    #[inline]
    fn render(self) -> Self::Rendered {
        Cow::borrowed(self).render()
    }
}

impl Html for String {
    type Rendered = RenderedText;

    #[inline]
    fn render(self) -> Self::Rendered {
        let text: Cow<'static, str> = Cow::owned(self);

        text.render()
    }
}

impl Html for std::borrow::Cow<'static, str> {
    type Rendered = RenderedText;

    #[inline]
    fn render(self) -> Self::Rendered {
        let text: Cow<'static, str> = self.into();

        text.render()
    }
}

impl Html for Cow<'static, str> {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        let node = document().create_text_node(&self).into();

        RenderedText {
            text: self,
            node,
        }
    }
}

impl Update<&'static str> for RenderedText {
    #[inline]
    fn update(&mut self, new: &'static str) {
        self.update(Cow::borrowed(new));
    }
}

impl Update<String> for RenderedText {
    #[inline]
    fn update(&mut self, new: String) {
        self.update(Cow::owned(new));
    }
}

impl Update<std::borrow::Cow<'static, str>> for RenderedText {
    #[inline]
    fn update(&mut self, new: std::borrow::Cow<'static, str>) {
        let new: Cow<'static, str> = new.into();

        self.update(new);
    }
}

impl Update<Cow<'static, str>> for RenderedText {
    fn update(&mut self, new: Cow<'static, str>) {
        if self.text != new {
            self.node.set_text_content(Some(&new));
            self.text = new;
        }
    }
}

fn bool_to_str(b: bool) -> &'static str {
    match b {
        true => "true",
        false => "false",
    }
}

impl Html for bool {
    type Rendered = RenderedText;

    fn render(self) -> Self::Rendered {
        bool_to_str(self).render()
    }
}

impl Update<bool> for RenderedText {
    fn update(&mut self, new: bool) {
        self.update(bool_to_str(new));
    }
}

macro_rules! impl_render_display {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Rendered = RenderedText;

                fn render(self) -> Self::Rendered {
                    self.to_string().render()
                }
            }

            impl Update<$t> for RenderedText {
                fn update(&mut self, new: $t) {
                    self.update(new.to_string())
                }
            }
        )*
    };
}

impl_render_display!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

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

        let node = document().create_element("div").unwrap().into();

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

        let node = document().create_element("div").unwrap().into();

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

        let node = document().create_element("div").unwrap().into();

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
