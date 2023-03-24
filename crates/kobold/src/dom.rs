//! Utilities for mounting elements in the DOM

use std::ops::Deref;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::value::FastDiff;
use crate::{util, Mountable, View};

#[derive(Clone)]
pub struct Element {
    kind: Kind,
    pub(crate) node: Node,
}

#[derive(Clone, Copy)]
enum Kind {
    Element,
    Fragment,
}

impl Deref for Element {
    type Target = JsValue;

    fn deref(&self) -> &JsValue {
        &self.node
    }
}

pub struct Fragment {
    el: Element,
    tail: Node,
}

impl Fragment {
    pub fn new() -> Self {
        let node = util::__kobold_fragment();
        let tail = util::__kobold_fragment_decorate(&node);
        Fragment {
            el: Element {
                kind: Kind::Fragment,
                node,
            },
            tail,
        }
    }

    pub fn append(&self, child: &JsValue) {
        util::__kobold_before(&self.tail, child);
    }
}

impl Deref for Fragment {
    type Target = Element;

    fn deref(&self) -> &Element {
        &self.el
    }
}

pub trait Text {
    fn to_text(&self) -> Node;

    fn set_text(&self, node: &Node);

    fn set_attr(&self, el: &JsValue);

    #[inline]
    fn no_diff(self) -> NoDiff<Self>
    where
        Self: Sized,
    {
        NoDiff(self)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NoDiff<T>(pub(crate) T);

impl<T> Deref for NoDiff<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> AsRef<T> for NoDiff<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Text for NoDiff<T>
where
    T: Text,
{
    fn to_text(&self) -> Node {
        self.0.to_text()
    }

    fn set_text(&self, node: &Node) {
        self.0.set_text(node);
    }

    fn set_attr(&self, el: &JsValue) {
        self.0.set_attr(el);
    }
}

impl Text for FastDiff<'_> {
    fn to_text(&self) -> Node {
        self.0.to_text()
    }

    fn set_text(&self, node: &Node) {
        self.0.set_text(node);
    }

    fn set_attr(&self, el: &JsValue) {
        self.0.set_attr(el);
    }
}

impl<T: Text + Copy> View for NoDiff<T> {
    type Product = Element;

    fn build(self) -> Self::Product {
        Element::new(self.to_text())
    }

    fn update(self, _: &mut Self::Product) {}
}

macro_rules! impl_text {
    ($($ty:ty),* [$make:ident, $update:ident, $set:ident]) => {
        $(
            impl Text for $ty {
                fn to_text(&self) -> Node {
                    util::$make(*self as _)
                }

                fn set_text(&self, node: &Node) {
                    util::$update(node, *self as _);
                }

                fn set_attr(&self, el: &JsValue) {
                    util::$set(el, *self as _);
                }
            }

            impl Text for &$ty {
                fn to_text(&self) -> Node {
                    (*self).to_text()
                }

                fn set_text(&self, node: &Node) {
                    (*self).set_text(node)
                }

                fn set_attr(&self, el: &JsValue) {
                    (*self).set_attr(el)
                }
            }
        )*
    };
}

impl Text for str {
    fn to_text(&self) -> Node {
        util::text_node(self)
    }

    fn set_text(&self, node: &Node) {
        util::set_text(node, self);
    }

    fn set_attr(&self, el: &JsValue) {
        util::set_attr(el, self);
    }
}

impl Text for &str {
    fn to_text(&self) -> Node {
        (*self).to_text()
    }

    fn set_text(&self, node: &Node) {
        (*self).set_text(node)
    }

    fn set_attr(&self, el: &JsValue) {
        (*self).set_attr(el)
    }
}

impl_text!(bool [text_node_bool, set_text_bool, set_attr_bool]);
impl_text!(i8, i16, i32, isize, u8, u16, u32, usize, f32, f64 [text_node_num, set_text_num, set_attr_num]);

pub trait LargeInt: Sized + Copy + PartialEq + 'static {
    type Downcast: TryFrom<Self> + Text;

    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R;
}

impl<S: LargeInt> Text for S {
    fn to_text(&self) -> Node {
        match S::Downcast::try_from(*self) {
            Ok(downcast) => downcast.to_text(),
            Err(_) => self.stringify(util::text_node),
        }
    }

    fn set_text(&self, node: &Node) {
        match S::Downcast::try_from(*self) {
            Ok(downcast) => downcast.set_text(node),
            Err(_) => self.stringify(|s| util::set_text(node, s)),
        }
    }

    fn set_attr(&self, el: &JsValue) {
        match S::Downcast::try_from(*self) {
            Ok(downcast) => downcast.set_attr(el),
            Err(_) => self.stringify(|s| util::set_attr(el, s)),
        }
    }
}

impl Element {
    pub fn new(node: Node) -> Self {
        Element {
            kind: Kind::Element,
            node,
        }
    }

    pub fn new_text(text: impl Text) -> Self {
        Self::new(text.to_text())
    }

    pub fn new_empty() -> Self {
        Self::new(util::__kobold_empty_node())
    }

    pub fn new_fragment_raw(node: Node) -> Self {
        util::__kobold_fragment_decorate(&node);

        Element {
            kind: Kind::Fragment,
            node,
        }
    }

    pub fn set_text(&self, text: impl Text) {
        text.set_text(&self.node);
    }

    pub fn anchor(&self) -> &JsValue {
        &self.node
    }

    pub fn js(&self) -> &JsValue {
        &self.node
    }

    pub fn replace_with(&self, new: &JsValue) {
        match self.kind {
            Kind::Element => util::__kobold_replace(&self.node, new),
            Kind::Fragment => util::__kobold_fragment_replace(&self.node, new),
        }
    }

    pub fn unmount(&self) {
        match self.kind {
            Kind::Element => util::__kobold_unmount(&self.node),
            Kind::Fragment => util::__kobold_fragment_unmount(&self.node),
        }
    }
}

impl Mountable for Element {
    type Js = Node;

    fn el(&self) -> &Element {
        self
    }
}

impl Drop for Element {
    fn drop(&mut self) {
        if let Kind::Fragment = self.kind {
            util::__kobold_fragment_drop(&self.node);
        }
    }
}
