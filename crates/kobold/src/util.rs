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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["document", "body"], js_name = appendChild)]
    pub(crate) fn append_body(node: &JsValue);
    #[wasm_bindgen(js_namespace = document, js_name = createTextNode)]
    pub(crate) fn text_node(t: &str) -> Node;
    #[wasm_bindgen(js_namespace = document, js_name = createTextNode)]
    pub(crate) fn text_node_num(t: f64) -> Node;
    #[wasm_bindgen(js_namespace = document, js_name = createTextNode)]
    pub(crate) fn text_node_bool(t: bool) -> Node;
}

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
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

    // `set_text` variants ----------------

    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text(el: &Node, t: &str);
    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text_num(el: &Node, t: f64);
    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text_bool(el: &Node, t: bool);

    // `set_attr` variants ----------------

    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr(el: &JsValue, a: &str, v: &str);
    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_num(el: &JsValue, a: &str, v: f64);
    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_bool(el: &JsValue, a: &str, v: bool);

    // provided attribute setters ----------------

    #[wasm_bindgen(js_name = "__kobold_checked")]
    pub(crate) fn checked(el: &Node, value: bool);
    #[wasm_bindgen(js_name = "__kobold_class_name")]
    pub(crate) fn class_name(el: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_href")]
    pub(crate) fn href(el: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_style")]
    pub(crate) fn style(el: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_value")]
    pub(crate) fn value(el: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_value")]
    pub(crate) fn value_num(el: &Node, value: f64);

    // ----------------

    #[wasm_bindgen(js_name = "__kobold_add_class")]
    pub(crate) fn add_class(el: &JsValue, value: &str);
    #[wasm_bindgen(js_name = "__kobold_remove_class")]
    pub(crate) fn remove_class(el: &JsValue, value: &str);
    #[wasm_bindgen(js_name = "__kobold_replace_class")]
    pub(crate) fn replace_class(el: &JsValue, old: &str, value: &str);
}
