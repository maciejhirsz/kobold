use kobold::prelude::*;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[derive(Debug)]
    struct Greeter {
        name: &'static str,
        count: u32,
        link: Link<Self>,
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

        fn create(props: Self::Properties, link: Link<Self>) -> Self {
            Self {
                name: props.name,
                count: 2,
                link,
            }
        }

        fn update(&mut self, props: Self::Properties) -> ShouldRender {
            self.name = props.name;

            true
        }

        fn handle(&mut self, msg: Msg) -> ShouldRender {
            match msg {
                Msg::Increment => self.count += 1,
                Msg::Decrement => self.count = self.count.saturating_sub(1),
            }

            true
        }
    }

    impl Greeter {
        fn render(&self) -> impl Html {
            let n = self.count;

            let inc = self.link.bind(|_| Msg::Increment);
            let dec = self.link.bind(|_| Msg::Decrement);

            html! {
                <div>
                    <h1 class="Greeter">"Hello "{ self.name }"!"</h1>
                    <p>
                        <button onclick={inc}>"+"</button>
                        { self.count }
                        <button onclick={dec}>"-"</button>
                    </p>
                    <p>
                        <strong>{ n }" + 2 = "{ n + 2 }</strong>
                    </p>
                    { for (0..n).map(|n| html! { <p>"Item #"{ n }</p> }) }
                </div>
            }
        }
    }

    kobold::start(html! {
        <Greeter name={"Bob"} />
    });
}
