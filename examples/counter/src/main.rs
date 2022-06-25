use kobold::prelude::*;

// To derive `Stateful` the component must also implement `PartialEq`.
#[derive(Stateful, PartialEq, Default)]
struct Counter {
    count: u32,
}

impl Counter {
    fn render(self) -> impl Html {
        self.stateful(|state, ctx| {
            let onclick = ctx.bind(|state, _event| state.count += 1);

            html! {
                <p>
                    "You clicked on the "
                    // `{onclick}` here is shorthand for `onclick={onclick}`
                    <button {onclick}>"Button"</button>
                    " "{ state.count }" times."
                </p>
            }
        })
    }
}

fn main() {
    kobold::start(html! {
        // The `..` notation fills in the rest of the component with values from the `Default` impl.
        <Counter ../>
    });
}
