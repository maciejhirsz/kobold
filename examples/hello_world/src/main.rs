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


struct User {
    name: String,
    email: String,
}
#[component]
fn user_row(user: &User) -> impl View + '_ {
    view! {
        <tr>
            // If `name` and `email` are always sent to the UI as
            // newly allocated `String`s, it's both safe and faster
            // to diff them by reference than value.
            <td>{ ref user.name }</td>
            <td>{ ref user.email }</td>
        </tr>
    }
}
