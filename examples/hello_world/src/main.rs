use kobold::prelude::*;

#[component]
fn Hello(name: &str) -> impl Html + '_ {
    html! {
        <h1>"Hello "{ name }"!"</h1>
    }
}

fn main() {
    kobold::start(html! {
        <Hello name="Kobold" />
    });
}
