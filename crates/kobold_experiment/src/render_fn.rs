use std::marker::PhantomData;

use crate::Html;
use crate::stateful::Link;

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
    pub fn new<'a, H>(render: fn(&'a S, &'a Link<S, P>) -> H) -> Self
    where
        H: Html<Product = P> + 'a,
    {
        RenderFn {
            ptr: render as usize,
            _marker: PhantomData,
        }
    }

    pub unsafe fn cast<'a, H>(self) -> fn(&'a S, &'a Link<S, P>) -> H
    where
        H: Html<Product = P> + 'a,
    {
        std::mem::transmute(self.ptr)
    }
}
