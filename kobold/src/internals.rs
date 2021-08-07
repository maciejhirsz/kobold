use crate::scope::{Scope, Weak};
use crate::traits::{Component, Html, MessageHandler, Mountable, Update};
use std::marker::PhantomData;
use wasm_bindgen::JsValue;
use web_sys::Event;

/// Wrapper containing proprs needed to build a component `T`, and its render method `R`.
pub struct WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
{
    props: T::Properties,
    /// Once returning `impl T` from trait methods is stable we can put the
    /// `render` method directly on the `Component` trait. Until then this
    /// solution is zero-cost since `R` is 0-sized.
    render: R,

    _phantom: PhantomData<H>,
}

impl<T, R, H> WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
{
    #[inline]
    pub fn new(props: T::Properties, render: R) -> Self {
        WrappedProperties {
            props,
            render,
            _phantom: PhantomData,
        }
    }
}

pub struct BuiltComponent<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
    H: Html,
{
    scope: Scope<ScopedComponent<T, R, H>>,
    node: JsValue,
}

pub struct ScopedComponent<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
    H: Html,
{
    component: T,
    render: R,
    built: H::Built,
}

impl<T, R, H> Html for WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
    H: Html,
{
    type Built = BuiltComponent<T, R, H>;

    #[inline]
    fn build(self) -> Self::Built {
        let scope = Scope::new_uninit();

        let render = self.render;
        let component = T::create(self.props);
        let built = (render)(&component, Link::new(scope.new_weak())).build();
        let node = built.js().clone();

        BuiltComponent {
            scope: scope.init(ScopedComponent {
                component,
                render,
                built,
            }),
            node,
        }
    }
}

impl<T, R, H> MessageHandler for ScopedComponent<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
    H: Html,
{
    type Component = T;

    fn handle(&mut self, message: <Self::Component as Component>::Message, link: Link<T>) {
        if self.component.handle(message) {
            // self.built.update((self.render)(&self.component, Link::new(self.new_weak())))
        }
    }
}

#[derive(Clone)]
pub struct Link<T: Component + ?Sized>(Weak<dyn MessageHandler<Component = T>>);

impl<T: Component> Link<T> {
    fn new(handler: Weak<impl MessageHandler<Component = T>>) -> Self {
        Link(handler.coerce())
    }

    pub fn bind(self, f: impl FnMut(&Event)) {}
}

impl<T, R, H> Mountable for BuiltComponent<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
    H: Html,
{
    fn js(&self) -> &JsValue {
        &self.node
    }
}

impl<T, R, H> Update<WrappedProperties<T, R, H>> for BuiltComponent<T, R, H>
where
    T: Component,
    R: Fn(&T, Link<T>) -> H + 'static,
    H: Html,
{
    #[inline]
    fn update(&mut self, new: WrappedProperties<T, R, H>) {
        let mut this = self.scope.borrow();

        if this.component.update(new.props) {
            let rendered = (new.render)(&this.component, Link::new(self.scope.new_weak()));
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
            fn render(&self, _: Link<Self>) -> impl Html {
                self.n
            }
        }

        impl Component for TestComponent {
            type Properties = u8;

            type Message = ();

            fn create(n: u8) -> Self {
                Self { n }
            }

            fn update(&mut self, new: u8) -> ShouldRender {
                self.n = new;
                true
            }
        }

        let wrapped = WrappedProperties::new(42_u8, TestComponent::render);

        assert_eq!(mem::size_of_val(&wrapped), mem::size_of::<u8>());
    }
}
