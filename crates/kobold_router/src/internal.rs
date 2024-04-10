use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
    #[wasm_bindgen(js_name = "changeRouteView")]
    pub(crate) fn change_route_view(view: &JsValue);

    #[wasm_bindgen(js_name = "setupPushStateEvent")]
    pub(crate) fn setup_push_state_event();
}
