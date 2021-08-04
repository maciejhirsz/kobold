use crate::util;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub type ShouldRender = bool;

pub trait Html: Sized {
    type Rendered: Update<Self> + Mountable;

    fn render(self) -> Self::Rendered;
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

pub trait Component: Sized {
    type Properties;

    fn create(props: Self::Properties) -> Self;

    fn update(&mut self, new: Self::Properties) -> ShouldRender;
}

pub trait HandleMessage<Message>: Component {
    fn handle(&mut self, message: Message);
}

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
