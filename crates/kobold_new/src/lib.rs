#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod ptr;
mod util;
mod value;
mod traits;
pub mod internals;

pub use traits::{Html, Mountable, Component};

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

            type Rendered<'r> = impl Html;

            fn init(self) -> Self {
                self
            }

            fn render(state: &Self::State) -> Self::Rendered<'_> {
                state.name.as_ref()
            }
        }

        start(ComponentInit::new(MyComponent { name: "Bob".into() }));
    }
}
