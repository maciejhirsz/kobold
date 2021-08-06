use crate::prelude::*;
use crate::context::Context;
use wasm_bindgen::JsValue;

/// Wrapper containing proprs needed to build a component `T`, and its render method `R`.
pub struct WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H,
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
    R: Fn(&T) -> H,
{
    #[inline]
    pub fn new(props: T::Properties, render: R) -> Self {
        WrappedProperties { props, render }
    }
}

pub struct BuiltComponent<T, H>
where
    T: Component,
    H: Html,
{
    component: Context<T>,
    built: H::Built,
}

impl<T, R, H> Html for WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H,
    H: Html,
{
    type Built = BuiltComponent<T, H>;

    #[inline]
    fn build(self) -> Self::Built {
        let context = Context::new_uninit();

        let component = T::create(self.props);
        let built = (self.render)(&component).build();

        let component = context.init(component);

        BuiltComponent {
            component,
            built,
        }
    }
}

impl<T, H> Mountable for BuiltComponent<T, H>
where
    T: Component,
    H: Html,
{
    fn js(&self) -> &JsValue {
        self.built.js()
    }
}

impl<T, R, H> Update<WrappedProperties<T, R, H>> for BuiltComponent<T, H>
where
    T: Component,
    R: Fn(&T) -> H,
    H: Html,
{
    #[inline]
    fn update(&mut self, new: WrappedProperties<T, R, H>) {
        let component = &mut *self.component.borrow();

        if component.update(new.props) {
            self.built.update((new.render)(component));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
