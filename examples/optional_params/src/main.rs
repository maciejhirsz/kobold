use kobold::prelude::*;

#[component(
    // Make `name` an optional parameter, defaults to `"Kobold"`
    name?: "Kobold",
    // Make `age` an optional parameter, use the `Default` value
    age?,
)]
fn greeter<'a>(name: &'a str, age: Option<u32>) -> impl View + 'a {
    let age = age.map(|age| view!(", you are "{ age }" years old"));

    view! {
        <p> "Hello "{ name }{ age }
    }
}

fn main() {
    kobold::start(view! {
        // Hello Kobold
        <!greeter>
        // Hello Alice
        <!greeter name="Alice">
        // Hello Bob, you are 42 years old
        <!greeter name="Bob" age={42}>
    });
}
