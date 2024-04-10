use kobold::dom::Mountable;
use kobold::internal::In;
use kobold::prelude::*;
use matchit::Match;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};

mod internal;

/// Routes type
type Routes = matchit::Router<Box<dyn Fn()>>;
type Params = HashMap<String, String>;

/// A web router for Kobold
pub struct Router {
    router: Rc<RefCell<Routes>>,
}

/// Get the current path via web_sys
pub fn get_path() -> String {
    web_sys::window()
        .expect_throw("no window")
        .location()
        .pathname()
        .expect_throw("no pathname")
}

///Error handling for [`get_param`](Router::get_param)
#[derive(Debug)]
pub enum ParamError {
    CouldNotFindParam,
    CouldNotParseParam,
    ParamsNotSet,
}

/// Implement of [Router]
impl Router {
    pub fn new() -> Self {
        let router = Rc::new(RefCell::new(matchit::Router::new()));
        Router { router }
    }

    /// Add a route to the router
    pub fn add_route<F>(&mut self, route: &str, view: F)
    where
        F: Fn() + 'static,
    {
        self.router
            .borrow_mut()
            .insert(route, Box::new(move || view()))
            .expect_throw("Failed to insert route");
    }

    /// Starts and hosts your web app with a router
    pub fn start(&mut self) {
        kobold::start(view! {
           <div id="routerView"></div>
        });

        let local_router = Rc::clone(&self.router);

        let window = web_sys::window().expect_throw("no window");
        //This is what decides what is render and triggered by pushState
        let conditonal_router_render: Closure<dyn FnMut()> = Closure::new(
            move || match local_router.borrow().at(get_path().as_str()) {
                Ok(Match {
                    value: render_view_fn,
                    params,
                }) => {
                    let history = web_sys::window()
                        .expect_throw("no window")
                        .history()
                        .expect_throw("no history");

                    let params = params
                        .iter()
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect::<Params>();

                    match serde_wasm_bindgen::to_value(&params) {
                        Ok(new_state) => {
                            history
                                .replace_state(&new_state, "")
                                .expect_throw("failed to replace state");
                        }
                        Err(_) => {}
                    }

                    //Runs the Fn() to render out the view
                    render_view_fn();
                }
                //TODO add ability to load your own 404 page. Possibly view a macro, or raw html
                Err(_) => start_route(view! {
                    <h1> "404" </h1>
                }),
            },
        );

        //Sets up a listener for pushState events
        internal::setup_push_state_event();
        window
            .add_event_listener_with_callback(
                "pushState",
                conditonal_router_render.as_ref().unchecked_ref(),
            )
            .unwrap();
        //runs same above closure for back, forward,and refresh buttons
        window.set_onpopstate(Some(conditonal_router_render.as_ref().unchecked_ref()));

        conditonal_router_render.forget();

        navigate(get_path().as_str())
    }
}

/// Start a route with a view
pub fn start_route(view: impl View) {
    use std::mem::MaybeUninit;
    use std::pin::pin;

    let product = pin!(MaybeUninit::uninit());
    let product = In::pinned(product, move |p| view.build(p));

    internal::change_route_view(product.js())
}

/// Navigate to a new path/route
pub fn navigate(path: &str) {
    web_sys::window()
        .expect_throw("no window")
        .history()
        .expect_throw("no history")
        .push_state_with_url(&JsValue::NULL, "", Some(path))
        .expect_throw("failed to push state");
}

/// Get the value of a parameter from the current route
pub fn get_param<T: std::str::FromStr>(key: &str) -> Result<T, ParamError> {
    let history_state = web_sys::window()
        .expect_throw("no window")
        .history()
        .expect_throw("no history")
        .state()
        .expect_throw("no state");

    match serde_wasm_bindgen::from_value::<Params>(history_state) {
        Ok(params) => match params.get(key) {
            Some(value) => match value.parse::<T>() {
                Ok(value) => Ok(value),
                Err(_) => Err(ParamError::CouldNotParseParam),
            },
            None => Err(ParamError::CouldNotFindParam),
        },
        Err(_) => Err(ParamError::ParamsNotSet),
    }
}

#[component(class?: "")]
// Creates a link needed for routing with kobold_router
pub fn link<'a>(route: &'a str, class: &'a str, children: impl View + 'a) -> impl View + 'a {
    let route = String::from(route);
    // TODO Not sure if clone is the best solution, but also need the route for the href tag for browser decoration
    let href = route.clone();
    //TODO work on implmenting a fence for event listeners
    let onclick = move |event: MouseEvent<_>| {
        navigate(&route);
        event.prevent_default();
    };

    view! {
        <a href={href} {class} {onclick}>{children}</a>
    }
}

/// Allows short hand for creating a fn
#[macro_export]
macro_rules! route_view {
    ($view:expr) => {
        || crate::start_route($view)
    };
}
