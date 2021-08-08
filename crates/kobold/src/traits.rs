use crate::util;
use crate::Link;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub type ShouldRender = bool;

pub trait Html: Sized + 'static {
    type Built: Update<Self> + Mountable;

    fn build(self) -> Self::Built;
}

pub trait Update<H> {
    fn update(&mut self, new: H);
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

pub trait Component: Sized + 'static {
    type Properties;

    type Message;

    fn create(props: Self::Properties, link: Link<Self>) -> Self;

    fn update(&mut self, new: Self::Properties) -> ShouldRender;

    fn handle(&mut self, msg: Self::Message) -> ShouldRender;
}

pub(crate) trait MessageHandler: 'static {
    type Message;

    fn handle(&mut self, msg: Self::Message);
}

// pub trait HandleMessage<Message>: Component {
//     fn handle(&mut self, message: Message);
// }

// pub trait StatelessComponent {}

// impl<T: StatelessComponent> Component for T {
//     type Properties = Self;

//     #[inline]
//     fn create(props: Self::Properties) -> Self {
//         props
//     }

//     #[inline]
//     fn update(&mut self, new: Self::Properties) -> ShouldRender {
//         *self = Self::create(new);
//         true
//     }
// }
