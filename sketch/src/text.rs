use crate::traits::{Html, Mountable, Update};
use crate::util;
use beef::Cow;
use web_sys::Node;

pub struct RenderedText {
    text: Cow<'static, str>,
    node: Node,
}

impl Mountable for RenderedText {
    fn node(&self) -> &Node {
        &self.node
    }
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
        let node = util::__sketch_text_node(self.as_ref());

        RenderedText { text: self, node }
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
            util::__sketch_update_text(&self.node, new.as_ref());
            self.text = new;
        }
    }
}
