use gloo_timers::callback::Interval;
use kobold::prelude::*;

#[component]
fn elapsed(seconds: u32) -> impl View {
    stateful(seconds, |seconds| {
        view! {
            <p>
                "Elapsed seconds: "{ seconds }" "
                <button onclick={do *seconds = 0}>"Reset"</button>
        }
    })
    .once(|signal| {
        // `signal` is an owned `Signal<u32>` and can be safely moved.
        //
        // `Interval` is returned here and will be safely dropped with the component.
        Interval::new(1000, move || {
            signal.update(|seconds| *seconds += 1);
        })
    })
}

fn main() {
    kobold::start(view! {
        <!elapsed seconds={0}>
    });
}
