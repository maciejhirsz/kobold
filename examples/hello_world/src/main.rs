use kobold::prelude::*;

// #[component]
// fn hello(name: &str) -> impl View + '_ {
//     view! {
//         // No need to close tags at the end of the macro
//         <h1> "Hello "{ name }"!"
//     }
// }

// fn main() {
//     kobold::start(view! {
//         <!hello name="Kobold">
//     });
// }

// Capture children into the argument `n`
// #[component(children: n)]
fn add_ten(n: i32) -> i32 {
    // integers implement `View` so they can be passed by value
    n + 10
}

mod add_ten {
    use super::*;
    #[allow(non_camel_case_types)]
    pub struct Props;
    pub const fn props() -> Props {
        Props {}
    }
    pub fn render_with(_: Props, n: i32) -> i32 {
        super::add_ten(n)
    }
    #[allow(non_camel_case_types)]
    impl Props {}
}

#[component]
fn test() -> impl View {
    "Test!"
}

fn main() {
    kobold::start(view! {
        <p>
            <!test>
            "Meaning of life is "
            <!add_ten>{ 32 }</!add_ten>
        </p>
    });
}
