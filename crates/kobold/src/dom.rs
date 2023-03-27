// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for mounting elements in the DOM

use std::ops::Deref;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::util;
use crate::value::IntoText;
use crate::Mountable;

/// A settable property of a DOM `Node`
pub trait Property<Abi> {
    fn set(self, this: &Node, value: Abi);
}

/// The `Node.textContent` property: <https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent>
pub struct TextContent;

impl Property<&str> for TextContent {
    fn set(self, this: &Node, value: &str) {
        util::set_text(this, value)
    }
}

impl Property<f64> for TextContent {
    fn set(self, this: &Node, value: f64) {
        util::set_text_num(this, value)
    }
}

impl Property<bool> for TextContent {
    fn set(self, this: &Node, value: bool) {
        util::set_text_bool(this, value)
    }
}

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
    type Target = Node;

    fn deref(&self) -> &Node {
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

/// A helper trait describing integers that might not fit in the JavaScript
/// number type and therefore might have to be passed as strings.
pub trait LargeInt: Sized + Copy + PartialEq + 'static {
    type Downcast: TryFrom<Self> + Into<f64> + IntoText;

    fn stringify<F: FnOnce(&str) -> R, R>(&self, f: F) -> R;
}

impl Element {
    pub fn new(node: Node) -> Self {
        Element {
            kind: Kind::Element,
            node,
        }
    }

    pub fn new_text(text: impl IntoText) -> Self {
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
