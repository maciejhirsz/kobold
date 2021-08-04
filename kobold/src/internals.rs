use crate::prelude::*;
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

pub struct RenderedComponent<T, H>
where
    T: Component,
    H: Html,
{
    component: T,
    rendered: H::Rendered,
}

impl<T, R, H> Html for WrappedProperties<T, R, H>
where
    T: Component,
    R: Fn(&T) -> H,
    H: Html,
{
    type Rendered = RenderedComponent<T, H>;

    #[inline]
    fn render(self) -> Self::Rendered {
        let component = T::create(self.props);
        let rendered = (self.render)(&component).render();

        RenderedComponent {
            component,
            rendered,
        }
    }
}

impl<T, H> Mountable for RenderedComponent<T, H>
where
    T: Component,
    H: Html,
{
    fn js(&self) -> &JsValue {
        self.rendered.js()
    }
}

impl<T, R, H> Update<WrappedProperties<T, R, H>> for RenderedComponent<T, H>
where
    T: Component,
    R: Fn(&T) -> H,
    H: Html,
{
    #[inline]
    fn update(&mut self, new: WrappedProperties<T, R, H>) {
        if self.component.update(new.props) {
            self.rendered.update((new.render)(&self.component));
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
