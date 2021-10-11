use crate::link::Link;
use crate::ptr::Prime;
use crate::traits::{Component, Html, MessageHandler, Mountable, Update};
use wasm_bindgen::JsValue;

/// Wrapper containing proprs needed to build a component `T`, and its render method `R`.
pub struct WrappedProperties<T, H>
where
    T: Component,
    // R: Fn(&T) -> H,
{
    props: T::Properties,
    /// Once returning `impl T` from trait methods is stable we can put the
    /// `render` method directly on the `Component` trait. Until then this
    /// solution is zero-cost since `R` is 0-sized.
    render: fn(&T) -> H,
}

impl<T, H> WrappedProperties<T, H>
where
    T: Component,
    // R: Fn(&T) -> H,
{
    #[inline]
    pub fn new(props: T::Properties, render: fn(&T) -> H) -> Self {
        WrappedProperties { props, render }
    }
}

pub struct BuiltComponent<T, B>
where
    // H: Html,
    Self: 'static,
{
    inner: Prime<InnerComponent<T, B>>,
    node: JsValue,
}

pub struct InnerComponent<T, B>
where
    Self: 'static,
{
    component: T,
    built: B,
}

impl<T, H> Html for WrappedProperties<T, H>
where
    T: Component,
    // R: Fn(&T) -> H,
    H: Html,
    Self: 'static,
{
    type Built = BuiltComponent<T, H::Built>;

    #[inline]
    fn build(self) -> Self::Built {
        let mut inner = Prime::new_uninit();

        let component = T::create(self.props, Link::new(inner.new_weak()));
        let built = (self.render)(&component).build();
        let node = built.js().clone();

        inner.init(InnerComponent {
            component,
            built,
        });

        BuiltComponent { inner, node }
    }
}

impl<T, B> MessageHandler for InnerComponent<T, B>
where
    T: Component,
{
    type Message = T::Message;

    fn handle(&mut self, message: Self::Message) {
        if self.component.handle(message) {
            panic!("What to do?");
            // self.built.update((self.render)(&self.component))
        }
    }
}

impl<T, B> Mountable for BuiltComponent<T, B> {
    fn js(&self) -> &JsValue {
        &self.node
    }
}

impl<T, H> Update<WrappedProperties<T, H>> for BuiltComponent<T, H::Built>
where
    T: Component,
    // R: Fn(&T) -> H,
    H: Html,
{
    #[inline]
    fn update(&mut self, new: WrappedProperties<T, H>) {
        let mut inner = self
            .inner
            .borrow()
            .expect("Component is currently borrowed by a Weak reference!");

        if inner.component.update(new.props) {
            let rendered = (new.render)(&inner.component);
            inner.built.update(rendered);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShouldRender;
    use std::mem;

    #[test]
    fn wrapped_component_is_zero_cost() {
        struct TestComponent {
            n: u8,
        }

        impl TestComponent {
            fn render(&self) -> impl Html {
                self.n
            }
        }

        impl Component for TestComponent {
            type Properties = u8;

            type Message = ();

            fn create(n: u8, _: Link<Self>) -> Self {
                Self { n }
            }

            fn update(&mut self, new: u8) -> ShouldRender {
                self.n = new;
                true
            }

            fn handle(&mut self, _: ()) -> ShouldRender {
                false
            }
        }

        let wrapped = WrappedProperties::new(42_u8, TestComponent::render);

        assert_eq!(mem::size_of_val(&wrapped), mem::size_of::<u8>());
    }
}
