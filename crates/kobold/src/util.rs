use wasm_bindgen::prelude::*;
use web_sys::Node;

use crate::dom::Element;
use crate::View;

pub struct Static<F = fn() -> Node>(pub F);

impl<F> View for Static<F>
where
    F: Fn() -> Node,
{
    type Product = Element;

    fn build(self) -> Element {
        Element::new(self.0())
    }

    fn update(self, _: &mut Element) {}
}

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
    pub(crate) fn __kobold_start(node: &JsValue);

    pub(crate) fn __kobold_append(parent: &Node, child: &JsValue);
    pub(crate) fn __kobold_before(node: &Node, insert: &JsValue);
    pub(crate) fn __kobold_unmount(node: &JsValue);
    pub(crate) fn __kobold_replace(old: &JsValue, new: &JsValue);

    pub(crate) fn __kobold_empty_node() -> Node;
    pub(crate) fn __kobold_fragment() -> Node;
    pub(crate) fn __kobold_fragment_decorate(f: &Node) -> Node;
    pub(crate) fn __kobold_fragment_append(f: &Node, c: &JsValue);
    pub(crate) fn __kobold_fragment_unmount(f: &Node);
    pub(crate) fn __kobold_fragment_replace(f: &Node, new: &JsValue);
    pub(crate) fn __kobold_fragment_drop(f: &Node);

    pub(crate) fn __kobold_text_node(t: &str) -> Node;
    #[wasm_bindgen(js_name = "__kobold_text_node_coerce")]
    pub(crate) fn __kobold_text_node_uint(t: u32) -> Node;
    #[wasm_bindgen(js_name = "__kobold_text_node_coerce")]
    pub(crate) fn __kobold_text_node_int(t: i32) -> Node;
    #[wasm_bindgen(js_name = "__kobold_text_node_coerce")]
    pub(crate) fn __kobold_text_node_float(t: f64) -> Node;
    #[wasm_bindgen(js_name = "__kobold_text_node_coerce")]
    pub(crate) fn __kobold_text_node_bool(t: bool) -> Node;
    pub(crate) fn __kobold_update_text(node: &Node, t: &str);
    #[wasm_bindgen(js_name = "__kobold_update_text_coerce")]
    pub(crate) fn __kobold_update_text_uint(node: &Node, t: u32);
    #[wasm_bindgen(js_name = "__kobold_update_text_coerce")]
    pub(crate) fn __kobold_update_text_int(node: &Node, t: i32);
    #[wasm_bindgen(js_name = "__kobold_update_text_coerce")]
    pub(crate) fn __kobold_update_text_float(node: &Node, t: f64);
    #[wasm_bindgen(js_name = "__kobold_update_text_coerce")]
    pub(crate) fn __kobold_update_text_bool(node: &Node, t: bool);

    pub(crate) fn __kobold_attr(name: &str, value: &str) -> Node;
    pub(crate) fn __kobold_attr_class(value: &str) -> Node;
    pub(crate) fn __kobold_attr_style(value: &str) -> Node;
    pub(crate) fn __kobold_attr_set(node: &JsValue, name: &str, value: &str) -> Node;
    pub(crate) fn __kobold_attr_update(node: &Node, value: &str);

    pub(crate) fn __kobold_attr_checked_set(el: &JsValue, value: bool);
    pub(crate) fn __kobold_class_set(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_add(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_remove(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_replace(el: &JsValue, old: &str, value: &str);
}
