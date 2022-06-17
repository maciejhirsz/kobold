#![feature(generic_associated_types)]
mod ptr;
mod util;
mod value;
mod traits;
pub mod internals;

pub use traits::{Html, Link, Mountable, Component};

pub fn start<H: Html>(html: H) {
    use std::mem::ManuallyDrop;

    let built = ManuallyDrop::new(html.build());

    util::__kobold_start(built.js());
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

        impl Component for MyComponent {
            type State = Self;

            fn init(self) -> Self {
                self
            }
        }

        impl MyComponent {
            fn render(&self, _link: &dyn Link<Self>) -> impl Html + '_ {
                self.name.as_ref()
            }
        }

        start(ComponentInit::new(
            MyComponent { name: "Bob".into() },
            MyComponent::render,
        ));
    }
}
