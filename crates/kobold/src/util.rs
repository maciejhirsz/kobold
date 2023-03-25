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
    pub(crate) fn set_text(el: &JsValue, t: &str);
    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text_num(el: &JsValue, t: f64);
    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text_bool(el: &JsValue, t: bool);

    // `set_attr` variants ----------------

    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr(el: &JsValue, a: &str, v: &str);
    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_num(el: &JsValue, a: &str, v: f64);
    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_bool(el: &JsValue, a: &str, v: bool);

    // `set_attr_value` variants ----------------

    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_value(el: &JsValue, v: &str);
    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_value_num(el: &JsValue, v: f64);
    #[wasm_bindgen(js_name = "__kobold_set_attr_value")]
    pub(crate) fn set_attr_value_bool(el: &JsValue, v: bool);

    // provided attribute constructors ----------------

    #[wasm_bindgen(js_name = "__kobold_attr_href")]
    pub(crate) fn href() -> Node;
    #[wasm_bindgen(js_name = "__kobold_attr_style")]
    pub(crate) fn style() -> Node;
    #[wasm_bindgen(js_name = "__kobold_attr_value")]
    pub(crate) fn value() -> Node;

    // ----------------

    #[wasm_bindgen(js_name = "__kobold_set_checked")]
    pub(crate) fn set_checked(el: &JsValue, value: bool);
    #[wasm_bindgen(js_name = "__kobold_set_class_name")]
    pub(crate) fn set_class_name(el: &JsValue, value: &str);
    #[wasm_bindgen(js_name = "__kobold_add_class")]
    pub(crate) fn add_class(el: &JsValue, value: &str);
    #[wasm_bindgen(js_name = "__kobold_remove_class")]
    pub(crate) fn remove_class(el: &JsValue, value: &str);
    #[wasm_bindgen(js_name = "__kobold_replace_class")]
    pub(crate) fn replace_class(el: &JsValue, old: &str, value: &str);
}
