use wasm_bindgen::prelude::*;
use web_sys::Node;

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
    pub(crate) fn __sketch_empty_node() -> Node;

    pub(crate) fn __sketch_text_node(t: &str) -> Node;

    pub(crate) fn __sketch_update_text(node: &Node, t: &str);

    pub(crate) fn __sketch_create_div() -> Node;

    pub(crate) fn __sketch_create_attr(name: &str, value: &str) -> Node;
    pub(crate) fn __sketch_create_attr_class(value: &str) -> Node;
    pub(crate) fn __sketch_create_attr_style(value: &str) -> Node;

    pub(crate) fn __sketch_update_attr(node: &Node, value: &str);
}
