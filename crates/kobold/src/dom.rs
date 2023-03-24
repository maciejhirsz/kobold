//! Utilities for mounting elements in the DOM

use std::ops::Deref;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::value::Stringify;
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
    fn into_node(self) -> Node;

    fn update(self, node: &Node);

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
        Element::new(self.into_node())
    }

    fn update(self, _: &mut Self::Product) {}
}

impl<S: Stringify> Text for S {
    fn into_node(self) -> Node {
        JsValue::from_str("Hello");
        self.stringify(util::__kobold_text_node)
    }

    fn update(self, node: &Node) {
        self.stringify(|s| util::__kobold_update_text(node, s))
    }
}

macro_rules! text {
    ($ty:ty as $as:ty, $make:ident, $update:ident) => {
        impl Text for $ty {
            fn into_node(self) -> Node {
                util::$make(self as $as)
            }

            fn update(self, node: &Node) {
                util::$update(node, self as $as);
            }
        }
    };
}

text!(&str as _, __kobold_text_node, __kobold_update_text);
text!(
    bool as _,
    __kobold_text_node_bool,
    __kobold_update_text_bool
);
text!(
    u8 as u32,
    __kobold_text_node_uint,
    __kobold_update_text_uint
);
text!(
    u16 as u32,
    __kobold_text_node_uint,
    __kobold_update_text_uint
);
text!(
    u32 as u32,
    __kobold_text_node_uint,
    __kobold_update_text_uint
);
text!(
    usize as u32,
    __kobold_text_node_uint,
    __kobold_update_text_uint
);
text!(i8 as i32, __kobold_text_node_int, __kobold_update_text_int);
text!(i16 as i32, __kobold_text_node_int, __kobold_update_text_int);
text!(i32 as i32, __kobold_text_node_int, __kobold_update_text_int);
text!(
    isize as i32,
    __kobold_text_node_int,
    __kobold_update_text_int
);
text!(
    f32 as f64,
    __kobold_text_node_float,
    __kobold_update_text_float
);
text!(
    f64 as f64,
    __kobold_text_node_float,
    __kobold_update_text_float
);

impl Element {
    pub fn new(node: Node) -> Self {
        Element {
            kind: Kind::Element,
            node,
        }
    }

    pub fn new_text(text: impl Text) -> Self {
        Self::new(text.into_node())
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
