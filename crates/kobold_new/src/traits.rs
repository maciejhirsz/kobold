use wasm_bindgen::JsValue;
use web_sys::Node;
use crate::util;

pub type ShouldRender = bool;

pub trait Component: Sized {
	type State: 'static;

    type Rendered<'a>: Html;

	fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> ShouldRender {
        *state = self.init();

        true
    }

    fn render(state: &Self::State) -> Self::Rendered<'_>;
}

pub trait Html {
    type Node: Mountable;

    fn build(self) -> Self::Node;

    fn update(self, built: &mut Self::Node);
}

pub trait Mountable: 'static {
    fn js(&self) -> &JsValue;

    fn mount(&self, parent: &Node) {
        util::__kobold_mount(parent, self.js());
    }

    fn unmount(&self, parent: &Node) {
        util::__kobold_unmount(parent, self.js());
    }
}

pub(crate) trait MessageHandler {
    type Message;

    fn handle(&mut self, msg: Self::Message);
}
