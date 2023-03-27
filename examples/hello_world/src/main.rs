use kobold::prelude::*;

#[component]
fn Hello(name: &str) -> impl View + '_ {
    view! {
        <h1>"Hello "{ static name }"!"</h1>
    }
}

fn main() {
    kobold::start(view! {
        <Hello name="Kobold" />
    });
}
