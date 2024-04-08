use crate::dom::Mountable;
use crate::internal::{In, Out};
use crate::{init, View};

use wasm_bindgen::JsValue;

pub struct Keyed<K, T> {
    pub(crate) key: K,
    pub(crate) item: T,
}

pub const fn with<K, V>(key: K, view: V) -> Keyed<K, V>
where
    K: Eq + 'static,
    V: View,
{
    Keyed { key, item: view }
}

impl<K, V> View for Keyed<K, V>
where
    K: Eq + 'static,
    V: View,
{
    type Product = Keyed<K, V::Product>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        p.in_place(|p| unsafe {
            init!(p.key = self.key);
            init!(p.item @ self.item.build(p));

            Out::from_raw(p)
        })
    }

    fn update(self, p: &mut Self::Product) {
        p.key = self.key;
        self.item.update(&mut p.item)
    }

    fn is_producer_of(&self, p: &Self::Product) -> bool {
        self.key == p.key
    }
}

impl<K, P> Mountable for Keyed<K, P>
where
    K: 'static,
    P: Mountable,
{
    type Js = P::Js;

    fn js(&self) -> &JsValue {
        self.item.js()
    }

    fn unmount(&self) {
        self.item.unmount()
    }

    fn replace_with(&self, new: &JsValue) {
        self.item.replace_with(new)
    }
}
