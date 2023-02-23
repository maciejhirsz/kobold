use gloo_timers::callback::Interval;
use kobold::prelude::*;

#[component]
fn Elapsed() -> impl Html {
    stateful(0, |seconds, ctx| {
        let onclick = ctx.bind(|seconds, _event| *seconds = 0);

        html! {
            <p>
                "Elapsed seconds: "{ *seconds }" "
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Reset"</button>
            </p>
        }
    })
    .then(|hook| {
        Interval::new(1000, move || {
            hook.update(|seconds| *seconds += 1).unwrap();
        })
        .forget();
    })
}

fn main() {
    kobold::start(html! {
        <Elapsed />
    });
}
