// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for mounting elements in the DOM

use std::ops::Deref;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::Node;

use crate::util;
use crate::Mountable;

pub trait Anchor {
    type Js: JsCast;
    type Anchor;

    fn anchor(&self) -> &Self::Anchor;
}

pub(crate) fn empty_node() -> Node {
    util::__kobold_empty_node()
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Fragment(Node);

impl From<Node> for Fragment {
    fn from(node: Node) -> Self {
        util::__kobold_fragment_decorate(&node);

        Fragment(node)
    }
}

impl AsRef<JsValue> for Fragment {
    fn as_ref(&self) -> &JsValue {
        self.0.as_ref()
    }
}

/// A settable property of a DOM `Node`
pub trait Property<Abi> {
    fn set(self, this: &Node, value: Abi);
}

/// The `Node.textContent` property: <https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent>
pub(crate) struct TextContent;

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

pub(crate) struct FragmentBuilder {
    fragment: Fragment,
    tail: Node,
}

impl FragmentBuilder {
    pub fn new() -> Self {
        let fragment = Fragment(util::__kobold_fragment());
        let tail = util::__kobold_fragment_decorate(&fragment.0);
        FragmentBuilder { fragment, tail }
    }

    pub fn append(&self, child: &JsValue) {
        util::__kobold_before(&self.tail, child);
    }
}

impl Deref for FragmentBuilder {
    type Target = Fragment;

    fn deref(&self) -> &Fragment {
        &self.fragment
    }
}

impl Mountable for Node {
    type Js = Node;

    fn js(&self) -> &JsValue {
        self
    }

    fn unmount(&self) {
        util::__kobold_unmount(self)
    }

    fn replace_with(&self, new: &JsValue) {
        util::__kobold_replace(self, new)
    }
}

impl Mountable for Fragment {
    type Js = Node;

    fn js(&self) -> &JsValue {
        &self.0
    }

    fn unmount(&self) {
        util::__kobold_fragment_unmount(&self.0)
    }

    fn replace_with(&self, new: &JsValue) {
        util::__kobold_fragment_replace(&self.0, new)
    }
}
