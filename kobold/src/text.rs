use crate::traits::{Html, Mountable, Update};
use crate::util;
use beef::Cow;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub struct BuiltText {
    text: Cow<'static, str>,
    node: Node,
}

impl Mountable for BuiltText {
    fn js(&self) -> &JsValue {
        &self.node
    }
}

impl Html for &'static str {
    type Built = BuiltText;

    #[inline]
    fn build(self) -> Self::Built {
        Cow::borrowed(self).build()
    }
}

impl Html for String {
    type Built = BuiltText;

    #[inline]
    fn build(self) -> Self::Built {
        let text: Cow<'static, str> = Cow::owned(self);

        text.build()
    }
}

impl Html for std::borrow::Cow<'static, str> {
    type Built = BuiltText;

    #[inline]
    fn build(self) -> Self::Built {
        let text: Cow<'static, str> = self.into();

        text.build()
    }
}
impl Html for Cow<'static, str> {
    type Built = BuiltText;

    fn build(self) -> Self::Built {
        let node = util::__kobold_text_node(self.as_ref());

        BuiltText { text: self, node }
    }
}

impl Update<&'static str> for BuiltText {
    #[inline]
    fn update(&mut self, new: &'static str) {
        self.update(Cow::borrowed(new));
    }
}

impl Update<String> for BuiltText {
    #[inline]
    fn update(&mut self, new: String) {
        self.update(Cow::owned(new));
    }
}

impl Update<std::borrow::Cow<'static, str>> for BuiltText {
    #[inline]
    fn update(&mut self, new: std::borrow::Cow<'static, str>) {
        let new: Cow<'static, str> = new.into();

        self.update(new);
    }
}

impl Update<Cow<'static, str>> for BuiltText {
    fn update(&mut self, new: Cow<'static, str>) {
        if self.text != new {
            util::__kobold_update_text(&self.node, new.as_ref());
            self.text = new;
        }
    }
}
