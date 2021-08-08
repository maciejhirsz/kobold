use wasm_bindgen::prelude::*;
use web_sys::Node;

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
    pub(crate) fn __kobold_start(node: &JsValue);

    pub(crate) fn __kobold_mount(parent: &Node, child: &JsValue);
    pub(crate) fn __kobold_unmount(parent: &Node, child: &JsValue);

    pub(crate) fn __kobold_empty_node() -> Node;

    pub(crate) fn __kobold_text_node(t: &str) -> Node;

    pub(crate) fn __kobold_update_text(node: &Node, t: &str);

    pub(crate) fn __kobold_create_div() -> Node;

    pub(crate) fn __kobold_create_attr(name: &str, value: &str) -> Node;
    pub(crate) fn __kobold_create_attr_class(value: &str) -> Node;
    pub(crate) fn __kobold_create_attr_style(value: &str) -> Node;

    pub(crate) fn __kobold_update_attr(node: &Node, value: &str);
}
