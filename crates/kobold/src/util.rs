use wasm_bindgen::prelude::*;
use web_sys::Node;

use crate::dom::Element;
use crate::{Html, Mountable};

pub struct Const<F>(F);

impl<F> Const<F> {
    pub const fn new(f: F) -> Const<F> {
        Const(f)
    }
}

impl<F: Fn() -> Node> Html for Const<F> {
    type Product = Element;

    fn build(self) -> Element {
        Element::new((self.0)())
    }

    fn update(self, _: &mut Element) {}
}

pub struct Static<T>(pub T);

impl<H: Html> Html for Static<H> {
    type Product = H::Product;

    fn build(self) -> H::Product {
        self.0.build()
    }

    fn update(self, _: &mut H::Product) {}
}

impl Html for fn() -> Node {
    type Product = StaticFnProduct;

    fn build(self) -> StaticFnProduct {
        StaticFnProduct {
            el: Element::new((self)()),
            render: self,
        }
    }

    fn update(self, p: &mut StaticFnProduct) {
        if p.render != self {
            let new = Element::new((self)());

            p.el.replace_with(new.js());
            p.el = new;
            p.render = self;
        }
    }
}

pub struct StaticFnProduct {
    el: Element,
    render: fn() -> Node,
}

impl Mountable for StaticFnProduct {
    fn el(&self) -> &Element {
        &self.el
    }
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
    pub(crate) fn __kobold_attr_set(node: &JsValue, name: &str, value: &str) -> Node;
    pub(crate) fn __kobold_attr_update(node: &Node, value: &str);

    pub(crate) fn __kobold_attr_checked_set(el: &JsValue, value: bool);
    pub(crate) fn __kobold_class_set(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_add(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_remove(el: &JsValue, value: &str);
    pub(crate) fn __kobold_class_replace(el: &JsValue, old: &str, value: &str);
}
