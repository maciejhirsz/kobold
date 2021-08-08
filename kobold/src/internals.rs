use crate::link::Link;
use crate::scope::Scope;
use crate::traits::{Component, Html, MessageHandler, Mountable, Update};
use wasm_bindgen::JsValue;

/// Wrapper containing proprs needed to build a component `T`, and its render method `R`.
pub struct WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
{
    props: T::Properties,
    /// Once returning `impl T` from trait methods is stable we can put the
    /// `render` method directly on the `Component` trait. Until then this
    /// solution is zero-cost since `R` is 0-sized.
    render: R,
}

impl<T, R, H> WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
{
    #[inline]
    pub fn new(props: T::Properties, render: R) -> Self {
        WrappedProperties { props, render }
    }
}

pub struct BuiltComponent<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
    H: Html,
{
    scope: Scope<ScopedComponent<T, R, H>>,
    node: JsValue,
}

pub struct ScopedComponent<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
    H: Html,
{
    component: T,
    render: R,
    built: H::Built,
}

impl<T, R, H> Html for WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
    H: Html,
{
    type Built = BuiltComponent<T, R, H>;

    #[inline]
    fn build(self) -> Self::Built {
        let mut scope = Scope::new_uninit();

        let render = self.render;
        let component = T::create(self.props, Link::new(scope.new_weak()));
        let built = render(&component).build();
        let node = built.js().clone();

        scope.init(ScopedComponent {
            component,
            render,
            built,
        });

        BuiltComponent { scope, node }
    }
}

impl<T, R, H> MessageHandler for ScopedComponent<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
    H: Html,
{
    type Message = T::Message;

    fn handle(&mut self, message: Self::Message) {
        if self.component.handle(message) {
            self.built.update((self.render)(&self.component))
        }
    }
}

impl<T, R, H> Mountable for BuiltComponent<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
    H: Html,
{
    fn js(&self) -> &JsValue {
        &self.node
    }
}

impl<T, R, H> Update<WrappedProperties<T, R, H>> for BuiltComponent<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H + 'static,
    H: Html,
{
    #[inline]
    fn update(&mut self, new: WrappedProperties<T, R, H>) {
        let mut this = self.scope.borrow();

        if this.component.update(new.props) {
            let rendered = (new.render)(&this.component);
            this.built.update(rendered);
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
