use kobold::prelude::*;

#[component(
    // Make `name` an optional parameter, default to `"Kobold"`
    name?: "Kobold",
    // Make `age` an optional parameter, use the `Default` trait
    age?,
)]
fn Greeter<'a>(name: &'a str, age: Option<u32>) -> impl View + 'a {
    let age = age.map(|age| view!(", you are "{ age }" years old"));

    view! {
        <p> "Hello "{ name }{ age }
    }
}

fn main() {
    kobold::start(view! {
        // Hello Kobold
        <Greeter />
        // Hello Alice
        <Greeter name="Alice" />
        // Hello Bob, you are 42 years old
        <Greeter name="Bob" age={42} />
    });
}
