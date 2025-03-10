use kobold::prelude::*;

#[component]
fn hello(name: &str) -> impl View + '_ {
    view! {
        // No need to close tags at the end of the macro
        <h1> "Hello "{ name }"!"
    }
}

fn main() {
    kobold::start(view! {
        <!hello name="Kobold">
    });
}
