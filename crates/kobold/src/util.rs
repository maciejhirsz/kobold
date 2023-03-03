use wasm_bindgen::prelude::*;
use web_sys::Node;

use crate::Html;

pub struct Static<T>(pub T);

impl<H: Html> Html for Static<H> {
    type Product = H::Product;

    fn build(self) -> H::Product {
        self.0.build()
    }

    fn update(self, _: &mut H::Product) {}
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

    pub(crate) fn __kobold_update_text(node: &Node, t: &str);

    pub(crate) fn __kobold_attr(name: &str, value: &str) -> Node;
    pub(crate) fn __kobold_attr_class(value: &str) -> Node;
    pub(crate) fn __kobold_attr_style(value: &str) -> Node;
    pub(crate) fn __kobold_attr_update(node: &Node, value: &str);

    pub(crate) fn __kobold_attr_checked_set(el: &JsValue, value: bool);
    pub(crate) fn __kobold_class_set(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_add(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_remove(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_replace(el: &JsValue, old: &str, value: &str);
}
