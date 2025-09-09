// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Kobold internals and types used by the [`view!`](crate::view) macro.

use wasm_bindgen::prelude::*;
use web_sys::Node;

use crate::View;

/// Wrapper that turns `extern` precompiled JavaScript functions into [`View`]s.
#[repr(transparent)]
pub struct Precompiled<F>(pub F);

/// Helper function used by the [`view!`](crate::view) macro to provide type hints for
/// event listeners.
#[inline]
pub const fn fn_type_hint<T, F: Fn(&T)>(f: F) -> F {
    f
}

impl<F> View for Precompiled<F>
where
    F: Fn() -> Node,
{
    type Product = Node;

    fn build(self) -> Node {
        self.0()
    }

    fn update(self, _: &mut Node) {}
}

#[wasm_bindgen]
extern "C" {
    pub(crate) type UnsafeNode;

    // dom manipulation ----------------

    #[wasm_bindgen(method, js_name = "before")]
    pub(crate) fn append_before(this: &UnsafeNode, insert: &JsValue);
    #[wasm_bindgen(method, js_name = "remove")]
    pub(crate) fn unmount(this: &UnsafeNode);
    #[wasm_bindgen(method, js_name = "replaceWith")]
    pub(crate) fn replace(this: &UnsafeNode, new: &JsValue);

    // `set_text` variants ----------------

    #[wasm_bindgen(method, setter, js_name = "textContent")]
    pub(crate) fn set_text(this: &UnsafeNode, t: &str);
    #[wasm_bindgen(method, setter, js_name = "textContent")]
    pub(crate) fn set_text_num(this: &UnsafeNode, t: f64);
    #[wasm_bindgen(method, setter, js_name = "textContent")]
    pub(crate) fn set_text_bool(this: &UnsafeNode, t: bool);

    // `set_attr` variants ----------------

    #[wasm_bindgen(method, js_name = "setAttribute")]
    pub(crate) fn set_attr(this: &UnsafeNode, a: &str, v: &str);
    #[wasm_bindgen(method, js_name = "setAttribute")]
    pub(crate) fn set_attr_num(this: &UnsafeNode, a: &str, v: f64);
    #[wasm_bindgen(method, js_name = "setAttribute")]
    pub(crate) fn set_attr_bool(this: &UnsafeNode, a: &str, v: bool);

    // provided attribute setters ----------------

    #[wasm_bindgen(method, setter, js_name = "className")]
    pub(crate) fn class_name(this: &UnsafeNode, value: &str);
    #[wasm_bindgen(method, setter, js_name = "innerHTML")]
    pub(crate) fn inner_html(this: &UnsafeNode, value: &str);
    #[wasm_bindgen(method, setter, js_name = "href")]
    pub(crate) fn href(this: &UnsafeNode, value: &str);
    #[wasm_bindgen(method, setter, js_name = "style")]
    pub(crate) fn style(this: &UnsafeNode, value: &str);
    #[wasm_bindgen(method, setter, js_name = "value")]
    pub(crate) fn value(this: &UnsafeNode, value: &str);
    #[wasm_bindgen(method, setter, js_name = "value")]
    pub(crate) fn value_num(this: &UnsafeNode, value: f64);
}

pub(crate) fn obj(node: &Node) -> &UnsafeNode {
    node.unchecked_ref()
}

mod hidden {
    use crate::runtime::{EventId, trigger};

    use super::wasm_bindgen;

    #[wasm_bindgen(js_name = "koboldTrigger")]
    pub fn kobold_trigger(eid: u32, event: web_sys::Event) {
        trigger(EventId(eid), event);
    }
}

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
    #[wasm_bindgen(js_name = appendBody)]
    pub(crate) fn append_body(node: &JsValue);
    #[wasm_bindgen(js_name = createTextNode)]
    pub(crate) fn text_node(t: &str) -> Node;
    #[wasm_bindgen(js_name = createTextNode)]
    pub(crate) fn text_node_num(t: f64) -> Node;
    #[wasm_bindgen(js_name = createTextNode)]
    pub(crate) fn text_node_bool(t: bool) -> Node;

    #[wasm_bindgen(js_name = "emptyNode")]
    pub(crate) fn empty_node() -> Node;
    #[wasm_bindgen(js_name = "fragment")]
    pub(crate) fn fragment() -> Node;
    #[wasm_bindgen(js_name = "fragmentDecorate")]
    pub(crate) fn fragment_decorate(f: &Node) -> Node;
    #[wasm_bindgen(js_name = "fragmentUnmount")]
    pub(crate) fn fragment_unmount(f: &Node);
    #[wasm_bindgen(js_name = "fragmentReplace")]
    pub(crate) fn fragment_replace(f: &Node, new: &JsValue);

    // provided attribute setters ----------------

    #[wasm_bindgen(js_name = "setChecked")]
    pub(crate) fn checked(node: &Node, value: bool);

    // ----------------

    #[wasm_bindgen(js_name = "addClass")]
    pub(crate) fn add_class(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "removeClass")]
    pub(crate) fn remove_class(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "replaceClass")]
    pub(crate) fn replace_class(node: &Node, old: &str, value: &str);
    #[wasm_bindgen(js_name = "toggleClass")]
    pub(crate) fn toggle_class(node: &Node, class: &str, value: bool);

    // ----------------

    #[wasm_bindgen(js_name = "makeEventHandler")]
    pub(crate) fn make_event_handler(eid: u32) -> JsValue;
    #[wasm_bindgen(js_name = "handlePopState")]
    pub(crate) fn handle_pop_state();
    #[wasm_bindgen(js_name = "getPath")]
    pub(crate) fn get_path() -> String;
}
