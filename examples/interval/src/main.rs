use gloo_timers::callback::Interval;
use kobold::prelude::*;

// To derive `Stateful` the component must also implement `PartialEq`.
#[derive(Stateful, PartialEq, Default)]
struct Elapsed {
    seconds: u32,
}

impl Elapsed {
    fn render(self) -> impl Html {
        self.stateful(|state, ctx| {
            let onclick = ctx.bind(|state, _event| state.seconds = 0);

            html! {
                <p>
                    "Elapsed seconds: "{ state.seconds }" "
                    // `{onclick}` here is shorthand for `onclick={onclick}`
                    <button {onclick}>"Reset"</button>
                </p>
            }
        })
        .then(|hook| {
            Interval::new(1000, move || {
                hook.update(|state| state.seconds += 1).unwrap();
            })
            .forget();
        })
    }
}

fn main() {
    kobold::start(html! {
        // The `..` notation fills in the rest of the component with values from the `Default` impl.
        <Elapsed ../>
    });
}
