use wasm_bindgen::JsValue;
use web_sys::Node;

mod util;
mod stateful;

pub use stateful::stateful;

pub trait Html: Sized {
    type Product: Mountable;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product);
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

pub type ShouldRender = bool;


struct Counter {
    n: i32,
}

impl Counter {
    pub fn render(self) -> impl Html {
        stateful(self.n, |state, link| {
            let inc = link.bind(|n| *n += 1);
            let dec = link.bind(|n| *n -= 1);

            *state
        })
    }
}

impl Html for i32 {
    type Product = i32;

    fn build(self) -> Self::Product {
        self
    }

    fn update(self, p: &mut Self::Product) {
        *p = self;
    }
}

impl Html for &str {
    type Product = String;

    fn build(self) -> Self::Product {
        self.into()
    }

    fn update(self, p: &mut Self::Product) {
        p.clear();
        p.push_str(self);
    }
}

impl Mountable for i32 {
    fn js(&self) -> &JsValue {
        panic!()
    }
}

impl Mountable for String {
    fn js(&self) -> &JsValue {
        panic!()
    }
}
