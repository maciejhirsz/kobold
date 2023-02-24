use std::marker::PhantomData;

use crate::stateful::Context;
use crate::Html;

/// Magic wrapper for render function that allows us to store it with a 'static
/// lifetime, without the lifetime on return type getting in the way
pub struct RenderFn<S, P> {
    ptr: usize,
    _marker: PhantomData<(S, P)>,
}

impl<S, P> Clone for RenderFn<S, P> {
    fn clone(&self) -> Self {
        RenderFn {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<S, P> Copy for RenderFn<S, P> {}

impl<S, P> RenderFn<S, P> {
    pub fn new<'a, H>(render: fn(&'a Context<S>) -> H) -> Self
    where
        H: Html<Product = P> + 'a,
    {
        RenderFn {
            ptr: render as usize,
            _marker: PhantomData,
        }
    }

    /// This is _mostly_ a safe call as long as the `H` type is the same
    /// `H` that was used in `new`. Since two different types can implement
    /// `Html` with the same `Product` associated type this needs to be unsafe.
    pub unsafe fn cast<'a, H>(self) -> fn(&'a Context<S>) -> H
    where
        H: Html<Product = P> + 'a,
    {
        std::mem::transmute(self.ptr)
    }
}
