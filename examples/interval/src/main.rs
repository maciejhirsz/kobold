use gloo_timers::callback::Interval;
use kobold::prelude::*;

#[component]
fn Elapsed() -> impl View {
    stateful(0_u32, |seconds| {
        bind! { seconds:
            let onclick = move |_| *seconds = 0;
        }

        view! {
            <p>
                "Elapsed seconds: "{ seconds }" "
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Reset"</button>
        }
    })
    .once(|hook| {
        Interval::new(1000, move || {
            hook.update(|seconds| *seconds += 1);
        })
        .forget();
    })
}

fn main() {
    kobold::start(view! {
        <Elapsed />
    });
}
