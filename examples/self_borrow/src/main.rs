use kobold_new::Html;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[derive(Debug)]
    struct Greeter {
        name: String,
    }

    // struct GreeterProps {
    //     name: &'static str,
    // }

    // impl Component for Greeter {
    //     type Properties = GreeterProps;

    //     type Message = ();

    //     fn create(props: Self::Properties, link: Link<Self>) -> Self {
    //         Self {
    //             name: props.name.into(),
    //         }
    //     }

    //     fn update(&mut self, props: Self::Properties) -> ShouldRender {
    //         self.name = props.name.into();

    //         true
    //     }

    //     fn handle(&mut self, msg: ()) -> ShouldRender {
    //         false
    //     }
    // }

    impl Greeter {
        fn render<'a>(&'a self) -> impl Html + 'a {
            html! {
                <div>
                    "Should be able to borrow a reference to string out:"{ self.name.as_str() }
                </div>
            }
        }
    }

    // kobold::start(html! {
    //     <Greeter name={"Bob"} />
    // });
    kobold::start(kobold::internals::WrappedProperties::new(GreeterProps { name: "Bob" }, Greeter::render))
}
