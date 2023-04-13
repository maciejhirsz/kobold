use gloo_timers::callback::Interval;
use kobold::prelude::*;

#[component(seconds?)]
fn Elapsed(seconds: u32) -> impl View {
    stateful(seconds, |seconds| {
        bind! {
            seconds:

            let onclick = move |_| *seconds = 0;
        }

        view! {
            <p>
                "Elapsed seconds: "{ seconds }" "
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Reset"</button>
        }
    })
    .once(|signal| {
        Interval::new(1000, move || {
            signal.update(|seconds| *seconds += 1);
        })
        .forget();
    })
}

fn main() {
    kobold::start(view! {
        <Elapsed seconds={0} />
    });
}
