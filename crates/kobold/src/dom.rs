//! Utilities for mounting elements in the DOM

use std::ops::Deref;

use wasm_bindgen::JsValue;
use web_sys::Node;

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

pub trait Text: Sized {
    fn into_text(self) -> Node;

    fn update(self, node: &Node);

    fn set_attr(self, el: &JsValue);

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

impl<T: Text + Copy> View for NoDiff<T> {
    type Product = Element;

    fn build(self) -> Self::Product {
        Element::new(self.into_text())
    }

    fn update(self, _: &mut Self::Product) {}
}

macro_rules! impl_text {
    ($make:ident, $update:ident, $set:ident; $($ty:ty),*) => {
        $(
            impl Text for $ty {
                fn into_text(self) -> Node {
                    util::$make(self as _)
                }

                fn update(self, node: &Node) {
                    util::$update(node, self as _);
                }

                fn set_attr(self, el: &JsValue) {
                    util::$set(el, self as _);
                }
            }
        )*
    };
}

impl_text!(text_node, set_text, set_attr; &str);
impl_text!(text_node_bool, set_text_bool, set_attr_bool; bool);
impl_text!(text_node_u32, set_text_u32, set_attr_u32; u8, u16, u32, usize);
impl_text!(text_node_i32, set_text_i32, set_attr_i32; i8, i16, i32, isize);
impl_text!(text_node_f64, set_text_f64, set_attr_f64; f32, f64);

pub trait LargeInt: Sized + Copy {
    type Downcast: TryFrom<Self> + Text;

    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R;
}

impl<S: LargeInt> Text for S {
    fn into_text(self) -> Node {
        match S::Downcast::try_from(self) {
            Ok(downcast) => downcast.into_text(),
            Err(_) => self.stringify(util::text_node),
        }
    }

    fn update(self, node: &Node) {
        match S::Downcast::try_from(self) {
            Ok(downcast) => downcast.update(node),
            Err(_) => self.stringify(|s| util::set_text(node, s)),
        }
    }

    fn set_attr(self, _el: &JsValue) {
        todo!();
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
        Self::new(text.into_text())
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
        text.update(&self.node);
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
