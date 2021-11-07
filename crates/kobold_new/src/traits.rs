use wasm_bindgen::JsValue;
use web_sys::Node;
use crate::util;

pub trait Component {
	type State: 'static;

	fn init(self) -> Self::State;
}

pub trait Html<'a> {
    type Node: Mountable;

    fn build(self) -> Self::Node;

    fn update(self, built: &mut Self::Node);
}

pub trait Mountable {
    fn js(&self) -> &JsValue;

    fn mount(&self, parent: &Node) {
        util::__kobold_mount(parent, self.js());
    }

    fn unmount(&self, parent: &Node) {
        util::__kobold_unmount(parent, self.js());
    }
}
