use kobold::prelude::*;
use kobold::View;
use kobold_router::get_param;
use kobold_router::link;
use kobold_router::route_view;
use kobold_router::start_route;
use kobold_router::Router;
use wasm_bindgen::JsValue;
use web_sys::console::error_1;
use web_sys::HtmlInputElement;

#[component]
fn inventory() -> impl View + 'static {
    let attempt_to_get_id = get_param::<usize>("id");

    let id = match attempt_to_get_id {
        Ok(id) => Some(id),
        Err(err) => {
            error_1(&JsValue::from_str(&format!("{:?}", err)));
            None
        }
    };

    view! {
        <!id_listing id={ id }>
    }
}

#[component]
fn id_listing(id: Option<usize>) -> impl View + 'static {
    view! {
        <h1>  "ID: "{ id } </h1>
    }
}

#[component]
fn router_example<'a>(state: &'a Hook<State>, route_number: &'a str) -> impl View + 'a {
    let onchange = event!(|state, e: Event<HtmlInputElement>| {
        let input = e.current_target();
        match input.value().parse::<usize>() {
            Ok(id) => state.update_inventory(id),
            Err(_) => error_1(&JsValue::from_str("Could not parse input value")),
        }
    });

    view! {
        <h1> "This route number "{ route_number }"!"</h1>
        <br>
        <!link route={"/one"} class={"styled-link"}>"Click to go to route one"</!link>
        <br>
        <!link route={"/two"}>"Click to go to route two"</!link>
        <br>
        <span> "Enter an inventory id"</span>
        <br>
        <input type="number" {onchange}>
        <br>
        <!link route={ref state.inventory_url}>{ref state.inventory_url}</!link>


    }
}

fn route_one(state: &Hook<State>) -> impl View + '_ {
    view! {
        <div>
            <!router_example route_number="one" state={state}>
        </div>
    }
}

fn route_two(state: &Hook<State>) -> impl View + '_ {
    view! {
        <div>
            <!router_example route_number="two" state={state}>
        </div>
    }
}

fn get_router() -> Router {
    let mut router = Router::new();
    router.add_route(
        "/",
        route_view!(view! {
            <h1>{"Welcome to the router example!"}</h1>
            <!link route={"/one"}>"View your first route here!"</!link>

        }),
    );

    router.add_route("/one", route_view!(stateful(State::default, route_one)));
    router.add_route("/two", route_view!(stateful(State::default, route_two)));
    router.add_route("/inventory/{id}", route_view!(view!(<!inventory>)));

    router
}

fn main() {
    let mut router = get_router();
    router.start();
}

struct State {
    inventory: Option<usize>,
    inventory_url: String,
}

impl State {
    pub fn update_inventory(&mut self, id: usize) {
        self.inventory = Some(id);
        self.inventory_url = format!("/inventory/{}", id);
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            inventory: None,
            inventory_url: String::from("No link yet"),
        }
    }
}
