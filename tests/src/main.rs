use web_sys::Event;
use kobold::prelude::*;
use kobold::Node;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[derive(Debug)]
    struct Greeter {
        name: &'static str,
        count: i32,
    }

    struct GreeterProps {
        name: &'static str,
    }

    enum Msg {
        Increment,
        Decrement,
    }

    impl Component for Greeter {
        type Properties = GreeterProps;

        type Message = Msg;

        fn create(props: Self::Properties) -> Self {
            Self {
                name: props.name,
                count: 0,
            }
        }

        fn update(&mut self, props: Self::Properties) -> ShouldRender {
            self.name = props.name;

            true
        }

        fn handle(&mut self, msg: Msg) -> ShouldRender {
            self.count += match msg {
                Msg::Increment => 1,
                Msg::Decrement => -1,
            };

            true
        }
    }

    impl Greeter {
        fn render(&self, link: Link<Self>) -> impl Html {
            let inc = link.bind(|_| Msg::Increment);
            let dec = link.bind(|_| Msg::Decrement);

            html! {
                <div>
                    <h1 class="Greeter">"Hello "{ self.name }"!"</h1>
                    <button onclick={inc}>"+"</button>{ self.count }<button onclick={dec}>"-"</button>
                </div>
            }
        }
    }

    fn hello(name: &'static str, n: u64) -> impl Html {
        let numbers = (0..(n % 5) + 6).map(|n| html! { <p>{ n }</p> });

        let color_base = n * 8;
        let rgb = u32::from_be_bytes([
            0,
            (color_base % 256) as u8,
            ((color_base + 85) % 256) as u8,
            ((color_base + 171) % 256) as u8,
        ]);

        let style = format!("color: #{:06x}", rgb);

        let onclick = move |event: &web_sys::Event| {
            let log = format!("Clicked while n = {}", n);
            let log = JsValue::from_str(&log);

            web_sys::console::log_2(&log, event.as_ref());
        };

        html! {
            <div>
                <Greeter {name} />
                <p {style}>
                    <strong {onclick}>{ n }" + 2 = "{ n + 2 }</strong>
                </p>
                { for numbers }
            </div>
        }
    }

    let bob = hello("Bob", 2);

    let mut built = bob.build();

    let window = kobold::reexport::web_sys::window().expect("should have a window in this context");
    let document = window.document().expect("window should have a document");
    let body = document.body().expect("document should have a body");
    let body: &Node = body.as_ref();

    body.append_child(unsafe { std::mem::transmute(built.js()) }).unwrap();

    let mut i = 2;
    let a = Closure::wrap(Box::new(move || {
        i += 1;

        static NAMES: &[&str] = &["Bob", "Alice", "Maciej", "World"];

        let name = NAMES[(i as usize / 10) % NAMES.len()];

        built.update(hello(name, i));
    }) as Box<dyn FnMut()>);

    window
        .set_interval_with_callback_and_timeout_and_arguments_0(a.as_ref().unchecked_ref(), 100)
        .unwrap();

    a.forget();
}
