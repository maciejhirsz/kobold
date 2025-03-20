use kobold::prelude::*;

#[component]
fn hello(name: &str) -> impl View {
    view! {
        // No need to close tags at the end of the macro
        <h1>"Hello "{ name }"!"
    }
}

kobold::start!(|| {
    view! {
        <!hello name="Kobold">
    }
});
