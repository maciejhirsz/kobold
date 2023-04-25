// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Kobold internals and types used by the [`view!`](crate::view) macro.

use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use wasm_bindgen::prelude::*;
use web_sys::Node;

use crate::View;

/// Safe abstraction for initialize-in-place strategy employed by the `View::build` method.
#[must_use]
#[repr(transparent)]
pub struct In<'a, T>(&'a mut MaybeUninit<T>);

#[repr(transparent)]
pub struct Out<'a, T>(&'a mut T);

impl<'a, T> Out<'a, T> {
    /// Create a new `Out<T>` pointer from a raw pointer to `T`.
    ///
    /// # Safety
    ///
    /// Caller needs to guarantee that:
    ///
    /// 1. `raw` is initialized.
    /// 2. `raw` is a stable pointer.
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Out(&mut *raw)
    }

    /// Cast this pointer from `Out<T>` to `Out<U>`.
    ///
    /// # Safety
    ///
    /// Caller needs to guarantee safety as per usual rules of pointer casting, namely:
    ///
    /// 1. `T` and `U` must have the same size.
    /// 2. `T` and `U` must have the same memory layout.
    pub unsafe fn cast<U>(self) -> Out<'a, U> {
        Out(&mut *(self.0 as *mut T as *mut U))
    }
}

impl<T> Deref for Out<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<T> DerefMut for Out<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T> In<'a, T> {
    /// Cast this pointer from `In<T>` to `In<U>`.
    ///
    /// # Safety
    ///
    /// Caller needs to guarantee safety as per usual rules of pointer casting, namely:
    ///
    /// 1. `T` and `U` must have the same size.
    /// 2. `T` and `U` must have the same memory layout.
    pub unsafe fn cast<U>(self) -> In<'a, U> {
        In(&mut *(self.0 as *mut MaybeUninit<T> as *mut MaybeUninit<U>))
    }

    pub fn in_place<F>(self, f: F) -> Out<'a, T>
    where
        F: FnOnce(*mut T) -> Out<'a, T>,
    {
        f(self.0.as_mut_ptr())
    }

    /// Initialize raw pointer `raw` using a builder closure `f`.
    ///
    /// # Safety
    ///
    /// Caller must guarantee that:
    ///
    /// 1. `raw` is uninitialized (it will be written to).
    /// 2. `raw` is a stable pointer.
    pub unsafe fn raw<F>(raw: *mut T, f: F) -> Out<'a, T>
    where
        F: FnOnce(In<T>) -> Out<T>,
    {
        f(In(&mut *(raw as *mut MaybeUninit<T>)))
    }

    /// Initialize a pinned uninitialized data using a builder closure `f`.
    ///
    /// # Safety
    ///
    /// This method is safe, it can however leak memory if `pin` has already been initialized.
    pub fn pinned<F>(pin: Pin<&'a mut MaybeUninit<T>>, f: F) -> Out<'a, T>
    where
        F: FnOnce(In<T>) -> Out<T>,
    {
        f(In(unsafe { pin.get_unchecked_mut() }))
    }

    /// Replace previous value of `T` with a new value produced by a builder
    /// closure `f`. Returns the old value.
    pub fn replace<F>(at: &mut T, f: F) -> T
    where
        F: FnOnce(In<T>) -> Out<T>,
    {
        let at = unsafe { &mut *(at as *mut T as *mut MaybeUninit<T>) };
        let old = unsafe { at.assume_init_read() };
        let Out(_) = f(In(at));

        old
    }

    /// Initialize this pointer with some value of `T`.
    pub fn put(self, val: T) -> Out<'a, T> {
        Out(self.0.write(val))
    }
}

/// Initialize a field of a struct with some expression, see [`In::in_place`](In::in_place).
#[macro_export]
macro_rules! init {
    ($p:ident.$field:ident @ $then:expr) => {
        $crate::internal::In::raw(std::ptr::addr_of_mut!((*$p).$field), move |$p| $then)
    };
    ($p:ident.$field:ident = $val:expr) => {
        $crate::internal::In::raw(std::ptr::addr_of_mut!((*$p).$field), |$p| $p.put($val))
    };
}

/// Wrapper that turns `extern` precompiled JavaScript functions into [`View`](View)s.
#[repr(transparent)]
pub struct Precompiled<F>(pub F);

/// Helper function used by the [`view!`](crate::view) macro to provide type hints for
/// event listeners.
#[inline]
pub const fn fn_type_hint<T, F: FnMut(T)>(f: F) -> F {
    f
}

impl<F> View for Precompiled<F>
where
    F: Fn() -> Node,
{
    type Product = Node;

    fn build(self, p: In<Node>) -> Out<Node> {
        p.put(self.0())
    }

    fn update(self, _: &mut Node) {}
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["document", "body"], js_name = appendChild)]
    pub(crate) fn append_body(node: &JsValue);
    #[wasm_bindgen(js_namespace = document, js_name = createTextNode)]
    pub(crate) fn text_node(t: &str) -> Node;
    #[wasm_bindgen(js_namespace = document, js_name = createTextNode)]
    pub(crate) fn text_node_num(t: f64) -> Node;
    #[wasm_bindgen(js_namespace = document, js_name = createTextNode)]
    pub(crate) fn text_node_bool(t: bool) -> Node;
}

#[wasm_bindgen(js_name = "koboldCallback")]
pub fn kobold_callback(event: web_sys::Event, closure: *mut (), vcall: usize) {
    let vcall: fn(web_sys::Event, *mut ()) = unsafe { std::mem::transmute(vcall) };

    vcall(event, closure);
}

#[wasm_bindgen(module = "/js/util.js")]
extern "C" {
    pub(crate) fn __kobold_append(parent: &Node, child: &JsValue);
    pub(crate) fn __kobold_before(node: &Node, insert: &JsValue);
    pub(crate) fn __kobold_unmount(node: &JsValue);
    pub(crate) fn __kobold_replace(old: &JsValue, new: &JsValue);

    pub(crate) fn __kobold_empty_node() -> Node;
    pub(crate) fn __kobold_fragment() -> Node;
    pub(crate) fn __kobold_fragment_decorate(f: &Node) -> Node;
    pub(crate) fn __kobold_fragment_append(f: &Node, c: &JsValue);
    pub(crate) fn __kobold_fragment_unmount(f: &Node);
    pub(crate) fn __kobold_fragment_replace(f: &Node, new: &JsValue);

    // `set_text` variants ----------------

    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text(el: &Node, t: &str);
    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text_num(el: &Node, t: f64);
    #[wasm_bindgen(js_name = "__kobold_set_text")]
    pub(crate) fn set_text_bool(el: &Node, t: bool);

    // `set_attr` variants ----------------

    #[wasm_bindgen(js_name = "__kobold_set_attr")]
    pub(crate) fn set_attr(el: &JsValue, a: &str, v: &str);
    #[wasm_bindgen(js_name = "__kobold_set_attr")]
    pub(crate) fn set_attr_num(el: &JsValue, a: &str, v: f64);
    #[wasm_bindgen(js_name = "__kobold_set_attr")]
    pub(crate) fn set_attr_bool(el: &JsValue, a: &str, v: bool);

    // provided attribute setters ----------------

    #[wasm_bindgen(js_name = "__kobold_checked")]
    pub(crate) fn checked(node: &Node, value: bool);
    #[wasm_bindgen(js_name = "__kobold_class_name")]
    pub(crate) fn class_name(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_href")]
    pub(crate) fn href(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_style")]
    pub(crate) fn style(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_value")]
    pub(crate) fn value(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_value")]
    pub(crate) fn value_num(node: &Node, value: f64);

    // ----------------

    #[wasm_bindgen(js_name = "__kobold_add_class")]
    pub(crate) fn add_class(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_remove_class")]
    pub(crate) fn remove_class(node: &Node, value: &str);
    #[wasm_bindgen(js_name = "__kobold_replace_class")]
    pub(crate) fn replace_class(node: &Node, old: &str, value: &str);
    #[wasm_bindgen(js_name = "__kobold_toggle_class")]
    pub(crate) fn toggle_class(node: &Node, class: &str, value: bool);

    // ----------------

    #[wasm_bindgen(js_name = "__kobold_make_event_handler")]
    pub(crate) fn make_event_handler(closure: *mut (), vcall: usize) -> JsValue;
}
