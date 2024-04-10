use std::str::FromStr;

use kobold::dom::Mountable;
use kobold::internal::In;
use kobold::prelude::*;

use matchit::Match;
use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};

mod internal;

/// Routes type
type Routes = matchit::Router<Box<dyn Fn(Params)>>;

/// A web router for Kobold
pub struct Router {
    router: Routes,
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
}

/// Implement of [Router]
impl Router {
    pub fn new() -> Self {
        let router = matchit::Router::new();
        Router { router }
    }

    /// Add a route to the router
    pub fn add_route<F, V>(&mut self, route: &str, render: F)
    where
        F: Fn(Params) -> V + 'static,
        V: View,
    {
        self.router
            .insert(
                route,
                Box::new(move |params| {
                    let view = render(params);

                    start_route(view)
                }),
            )
            .expect_throw("Failed to insert route");
    }

    /// Starts and hosts your web app with a router
    pub fn start(self) {
        kobold::start(view! {
           <div id="routerView"></div>
        });

        //This is what decides what is render and triggered by pushState
        let conditonal_router_render: Closure<dyn FnMut()> =
            Closure::new(move || match self.router.at(get_path().as_str()) {
                Ok(Match {
                    value: render,
                    params,
                }) => {
                    //Runs the Fn() to render out the view
                    render(Params(params));
                }
                //TODO add ability to load your own 404 page. Possibly view a macro, or raw html
                Err(_) => start_route(view! {
                    <h1> "404" </h1>
                }),
            });

        let window = web_sys::window().expect_throw("no window");

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
fn start_route(view: impl View) {
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

pub struct Params<'p>(matchit::Params<'p, 'p>);

impl Params<'_> {
    pub fn get<T>(&self, key: &str) -> Result<T, ParamError>
    where
        T: FromStr,
    {
        match self.0.get(key) {
            Some(value) => match value.parse::<T>() {
                Ok(value) => Ok(value),
                Err(_) => Err(ParamError::CouldNotParseParam),
            },
            None => Err(ParamError::CouldNotFindParam),
        }
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
