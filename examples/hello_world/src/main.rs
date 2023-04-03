use kobold::prelude::*;

#[component]
fn Hello(name: &str) -> impl View + '_ {
    view! {
        // No need to close tags at the end of the macro
        <h1> "Hello "{ name }"!"
    }
}

fn main() {
    kobold::start(view! {
        <Hello name="Kobold" />
    });
}
