use kobold::prelude::*;
use kobold::Node;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn main() {
    struct Greeter {
        name: &'static str,
    }

    impl Component for Greeter {
        type Properties = Self;

        fn create(props: Self::Properties) -> Self {
            props
        }

        // fn update(&mut self, props: Self::Properties) -> ShouldRender {
        //     self.name = props.name;

        //     matches!(self.name, "Alice" | "Bob")
        // }
    }

    impl Greeter {
        fn render(&self) -> impl Html {
            html! {
                <h1 class="Greeter">"Hello "{ self.name }"!"</h1>
            }
        }
    }

    fn hello(name: &'static str, n: u64) -> impl Html {
        let numbers = (0..(n % 5) + 6).map(|n| html! { <p>{ n }</p> });

        let color_base = n * 8;
        let (r, g, b) = (
            (color_base % 256) as u8,
            ((color_base + 85) % 256) as u8,
            ((color_base + 171) % 256) as u8,
        );

        let style = format!("color: #{:02x}{:02x}{:02x}", r, g, b);

        html! {
            <div>
                <Greeter {name} />
                <p {style}><strong>{ n }" + 2 = "{ n + 2 }</strong></p>
                { for numbers }
            </div>
        }
    }

    let bob = hello("Bob", 2);

    let mut rendered = bob.render();

    let window = kobold::reexport::web_sys::window().expect("should have a window in this context");
    let document = window.document().expect("window should have a document");
    let body = document.body().expect("document should have a body");
    let body: &Node = body.as_ref();

    body.append_child(&rendered.node()).unwrap();

    let mut i = 2;
    let a = Closure::wrap(Box::new(move || {
        i += 1;

        static NAMES: &[&str] = &["Bob", "Alice", "Maciej", "World"];

        let name = NAMES[(i as usize / 10) % NAMES.len()];

        rendered.update(hello(name, i));
    }) as Box<dyn FnMut()>);

    window
        .set_interval_with_callback_and_timeout_and_arguments_0(a.as_ref().unchecked_ref(), 100)
        .unwrap();

    a.forget();
}
