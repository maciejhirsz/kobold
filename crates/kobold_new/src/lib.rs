mod util;
mod value;
mod traits;
mod internals;

pub use traits::{Html, Mountable, Component};


pub fn start<'a>(html: impl Html<'a>) {
    use std::mem::ManuallyDrop;

    let built = ManuallyDrop::new(html.build());

    util::__kobold_start(built.js());
}

pub fn render<'a, T>(render: impl internals::Render<'a, T>) {
    panic!();
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::internals::ComponentInit;

    #[test]
    fn builds() {
        struct MyComponent {
            name: String,
        }

        // impl Component for MyComponent {
        //     type State = Self;

        //     fn init(self) -> Self {
        //         self
        //     }
        // }

        impl MyComponent {
            fn render(&self) -> impl Html {
                self.name.as_ref()
            }
        }

        render(MyComponent::render);

        // start(ComponentInit {
        //     component: MyComponent { name: "Bob".into() },
        //     render: MyComponent::render,
        // })
    }
}