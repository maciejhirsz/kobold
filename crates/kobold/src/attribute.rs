// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Utilities for dealing with DOM attributes
use std::ops::Deref;

use web_sys::Node;

use crate::diff::{Diff, Ref, VString};
use crate::dom::Property;
use crate::internal;
use crate::value::Value as Text;

/// Arbitrary attribute: <https://developer.mozilla.org/en-US/docs/Web/API/Element/setAttribute>
pub struct AttributeName(str);

impl From<&str> for &AttributeName {
    fn from(attr: &str) -> Self {
        unsafe { &*(attr as *const _ as *const AttributeName) }
    }
}

impl Deref for AttributeName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl Property<&str> for &AttributeName {
    fn set(self, this: &Node, value: &str) {
        internal::set_attr(this, self, value)
    }
}

impl Property<f64> for &AttributeName {
    fn set(self, this: &Node, value: f64) {
        internal::set_attr_num(this, self, value)
    }
}

impl Property<bool> for &AttributeName {
    fn set(self, this: &Node, value: bool) {
        internal::set_attr_bool(this, self, value)
    }
}

macro_rules! attribute {
    ($(#[doc = $doc:literal] $name:ident [ $($util:ident: $abi:ty),* ])*) => {
        $(
            #[doc = $doc]
            pub struct $name;

            $(
                impl Property<$abi> for $name {
                    fn set(self, this: &Node, value: $abi) {
                        internal::$util(this, value)
                    }
                }
            )*
        )*
    }
}

/// The `Element.classList` property: <https://developer.mozilla.org/en-US/docs/Web/API/Element/classList>
pub struct Class;

attribute!(
    /// The `checked` attribute: <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#checked>
    Checked [checked: bool]
    /// The `className` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/Element/className>
    ClassName [class_name: &str]
    /// The `innerHTML` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML>
    InnerHtml [inner_html: &str]
    /// The `style` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/style>
    Style [style: &str]
    /// The `href` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/href>
    Href [href: &str]
    /// The `value` attribute: <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#value>
    Value [value: &str, value_num: f64]
    /// The `viewBox` SVG attribute: <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/viewBox>
    SvgViewBox [svg_view_box: &str]
    /// The `d` SVG attribute: <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/d>
    SvgPath [svg_path: &str]
);

pub trait Attribute<P> {
    type Product: 'static;

    fn build(self) -> Self::Product;

    fn build_in(self, prop: P, node: &Node) -> Self::Product;

    fn update_in(self, prop: P, node: &Node, memo: &mut Self::Product);
}

impl<P> Attribute<P> for String
where
    P: for<'a> Property<&'a str>,
{
    type Product = String;

    fn build(self) -> Self::Product {
        self
    }

    fn build_in(self, prop: P, node: &Node) -> Self::Product {
        self.set_prop(prop, node);
        self
    }

    fn update_in(self, prop: P, node: &Node, prod: &mut Self::Product) {
        if &self != prod {
            self.set_prop(prop, node);
            *prod = self;
        }
    }
}

impl<P> Attribute<P> for bool
where
    Self: Text<P>,
{
    /// `bool` attributes can have weird behavior, it's best to
    /// diff them in the DOM directly
    type Product = ();

    fn build(self) {}

    fn build_in(self, prop: P, node: &Node) {
        self.set_prop(prop, node);
    }

    fn update_in(self, prop: P, node: &Node, _: &mut ()) {
        self.set_prop(prop, node);
    }
}

macro_rules! impl_attribute_view {
    ($($ty:ty),*) => {
        $(
            impl<P> Attribute<P> for $ty
            where
                Self: Text<P>,
            {
                type Product = <Self as Diff>::Memo;

                fn build(self) -> Self::Product {
                    self.into_memo()
                }

                fn build_in(self, prop: P, node: &Node) -> Self::Product {
                    self.set_prop(prop, node);
                    self.into_memo()
                }

                fn update_in(self, prop: P, node: &Node, prod: &mut Self::Product) {
                    if self.diff(prod) {
                        self.set_prop(prop, node);
                    }
                }
            }
        )*
    };
}

impl_attribute_view!(&str, &String, &Ref<str>, &VString);
impl_attribute_view!(u8, u16, u32, u64, u128, usize, isize, i8, i16, i32, i64, i128, f32, f64);

#[inline]
fn debug_test_class(class: &str) {
    debug_assert!(
        class.find(' ').is_none(),
        "Class name cannot contain spaces, offending class: \"{class}\"",
    );
}

fn set_class(node: &Node, class: &str) {
    if !class.is_empty() {
        internal::add_class(node, class);
    }
}

fn diff_class(node: &Node, new: &str, old: &str) -> bool {
    match (new, old) {
        (new, old) if new == old => return false,
        (new, "") => internal::add_class(node, new),
        ("", old) => internal::remove_class(node, old),
        (new, old) => internal::replace_class(node, old, new),
    }
    true
}

impl<T> Attribute<Class> for T
where
    T: Diff<Memo = String> + AsRef<str>,
{
    type Product = String;

    fn build(self) -> String {
        debug_test_class(self.as_ref());
        self.into_memo()
    }

    fn build_in(self, _: Class, node: &Node) -> String {
        set_class(node, self.as_ref());
        self.build()
    }

    fn update_in(self, _: Class, node: &Node, old: &mut String) {
        if diff_class(node, self.as_ref(), old) {
            old.clear();
            old.push_str(self.as_ref());
        }
    }
}

impl Attribute<Class> for String {
    type Product = String;

    fn build(self) -> String {
        debug_test_class(self.as_ref());
        self
    }

    fn build_in(self, _: Class, node: &Node) -> String {
        set_class(node, self.as_ref());
        self
    }

    fn update_in(self, _: Class, node: &Node, old: &mut String) {
        if diff_class(node, self.as_ref(), old) {
            *old = self;
        }
    }
}

#[derive(Clone, Copy)]
pub struct OptionalClass {
    class: &'static str,
    on: bool,
}

impl AsRef<str> for OptionalClass {
    fn as_ref(&self) -> &str {
        if self.on {
            self.class
        } else {
            ""
        }
    }
}

impl OptionalClass {
    pub const fn new(class: &'static str, on: bool) -> Self {
        OptionalClass { class, on }
    }
}

impl Attribute<Class> for OptionalClass {
    type Product = bool;

    fn build(self) -> bool {
        debug_test_class(self.class);
        self.on
    }

    fn build_in(self, _: Class, node: &Node) -> bool {
        internal::toggle_class(node, self.class, self.on);
        self.on
    }

    fn update_in(self, _: Class, node: &Node, memo: &mut bool) {
        if self.on != *memo {
            internal::toggle_class(node, self.class, self.on);
            *memo = self.on;
        }
    }
}

impl Attribute<ClassName> for OptionalClass {
    type Product = bool;

    fn build(self) -> bool {
        debug_test_class(self.class);
        self.on
    }

    fn build_in(self, _: ClassName, node: &Node) -> bool {
        if self.on {
            internal::class_name(node, self.class);
        }
        self.on
    }

    fn update_in(self, _: ClassName, node: &Node, memo: &mut bool) {
        if self.on != *memo {
            internal::class_name(node, self.as_ref());
            *memo = self.on;
        }
    }
}
