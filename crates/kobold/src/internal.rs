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
///
/// ```
/// use kobold::internal::Container;
///
/// let boxed: Box<u32> = Container::boxed(|container| container.put(42));
/// ```
#[must_use]
#[repr(transparent)]
pub struct Pre<'a, T>(&'a mut MaybeUninit<T>);

#[repr(transparent)]
pub struct Mut<'a, T>(&'a mut T);

impl<'a, T> Mut<'a, T> {
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Mut(&mut *raw)
    }
}

impl<T> Deref for Mut<'_, T>
where
    T: Unpin,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<T> DerefMut for Mut<'_, T>
where
    T: Unpin,
{
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

#[repr(transparent)]
pub struct Field<T>(MaybeUninit<T>);

impl<T> Field<T> {
    // `MaybeUninit::uninit` is safe, however `Field` must
    // meet additional guarantees:
    //
    // 1. It's created inside a stable (pinned) memory.
    // 2. It's not derefed before it's initialized.
    pub const unsafe fn uninit() -> Self {
        Field(MaybeUninit::uninit())
    }

    /// Creates a new field with value `T`.
    ///
    /// # Safety
    ///
    /// You must guarantee that this is a structural field inside a struct
    /// that's being placed in stable memory.
    pub const unsafe fn new(val: T) -> Self {
        Field(MaybeUninit::new(val))
    }

    pub fn init<F>(&mut self, f: F)
    where
        F: FnOnce(Pre<T>) -> Mut<T>,
    {
        // This will leak memory if done more than once, but it is safe
        let Mut(_) = f(Pre(&mut self.0));
    }

    pub fn get_ref(&self) -> &T {
        // Safety: it's not possible to create an `Stable`
        // uninitialized `Stable` without unsafe code
        unsafe { self.0.assume_init_ref() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // Safety: it's not possible to create an `Stable`
        // uninitialized `Stable` without unsafe code
        unsafe { self.0.assume_init_mut() }
    }
}

impl<T> Unpin for Field<T> {}

impl<T> Drop for Field<T> {
    fn drop(&mut self) {
        // Safety: it's not possible to create an `Stable`
        // uninitialized `Stable` without unsafe code
        unsafe { self.0.assume_init_drop() }
    }
}

impl<'a, T> Pre<'a, T>
where
    T: Unpin,
{
    pub fn boxed<F>(f: F) -> Pin<Box<T>>
    where
        F: FnOnce(Pre<T>) -> Mut<T>,
    {
        // Use `Box::new_uninit` when it's stabilized
        // <https://github.com/rust-lang/rust/issues/63291>
        let mut boxed = Box::new(MaybeUninit::uninit());
        let Mut(_) = f(Pre(boxed.as_mut()));

        // ⚠️ Safety:
        // ==========
        //
        // Since `F` needs to produce a receipt, and the only way to do it
        // is by putting a `T` in the container, the `Box` is now guaranteed
        // to be initialized.
        //
        // `MaybeUninit` is `#[repr(transparent)]` so transmute is safe.
        //
        // Use `Box::assume_init` when it's stabilized
        // <https://github.com/rust-lang/rust/issues/63291>
        unsafe { std::mem::transmute(boxed) }
    }

    pub fn into_raw(self) -> *mut T {
        self.0.as_mut_ptr()
    }

    pub unsafe fn in_raw<F>(raw: *mut T, f: F) -> Mut<'a, T>
    where
        F: FnOnce(Pre<T>) -> Mut<T>,
    {
        f(Pre(&mut *(raw as *mut MaybeUninit<T>)))
    }

    pub fn pinned<F>(pin: Pin<&'a mut MaybeUninit<T>>, f: F) -> Mut<'a, T>
    where
        F: FnOnce(Pre<T>) -> Mut<T>,
    {
        f(Pre(pin.get_mut()))
    }

    pub fn replace<F>(at: &mut T, f: F) -> T
    where
        F: FnOnce(Pre<T>) -> Mut<T>,
        T: Unpin,
    {
        let at = unsafe { &mut *(at as *mut T as *mut MaybeUninit<T>) };
        let old = unsafe { at.assume_init_read() };
        let Mut(_) = f(Pre(at));

        old
    }

    pub fn put(self, val: T) -> Mut<'a, T> {
        Mut(self.0.write(val))
    }
}

#[macro_export]
macro_rules! init {
    ($p:ident.$field:ident @ $then:expr) => {
        $crate::internal::Pre::in_raw(std::ptr::addr_of_mut!((*$p).$field), move |$p| $then)
    };
    ($p:ident.$field:ident = $val:expr) => {
        std::ptr::addr_of_mut!((*$p).$field).write($val)
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

    fn build(self, p: Pre<Node>) -> Mut<Node> {
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
}
