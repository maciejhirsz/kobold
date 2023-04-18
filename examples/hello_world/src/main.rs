use kobold::prelude::*;

#[component]
fn Hello(name: &str) -> impl View + '_ {
    // No need to close tags at the end of the macro
    view! { <h1 aria-foo="bar"> "Hello "{ name }"!" }
}

fn main() {
    kobold::start(view! {
        <Hello name="Kobold" />
    });
}
