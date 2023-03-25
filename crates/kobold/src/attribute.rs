//! Utilities for dealing with DOM attributes
use web_sys::Node;

use crate::value::Value;
use crate::dom::Property;
use crate::diff::Diff;
use crate::util;

/// Arbitrary attribute: <https://developer.mozilla.org/en-US/docs/Web/API/Element/setAttribute>
pub type Attribute = &'static str;

impl Property<&str> for Attribute {
    fn set(self, this: &Node, value: &str) {
        util::set_attr(this, self, value)
    }
}

impl Property<f64> for Attribute {
    fn set(self, this: &Node, value: f64) {
        util::set_attr_num(this, self, value)
    }
}

impl Property<bool> for Attribute {
    fn set(self, this: &Node, value: bool) {
        util::set_attr_bool(this, self, value)
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
                        util::$util(this, value)
                    }
                }
            )*
        )*
    }
}

attribute!(
    /// The `checked` attribute: <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#checked>
    Checked [checked: bool]
    /// The `className` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/Element/className>
    ClassName [class_name: &str]
    /// The `style` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/style>
    Style [style: &str]
    /// The `href` attribute: <https://developer.mozilla.org/en-US/docs/Web/API/HTMLAnchorElement/href>
    Href [href: &str]
    /// The `value` attribute: <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#value>
    InputValue [value: &str, value_num: f64]
);

pub trait AttributeView<P> {
    type Product: 'static;

    fn build_in(self, prop: P, node: &Node) -> Self::Product;

    fn update_in(self, prop: P, node: &Node, memo: &mut Self::Product);
}

impl<P, T> AttributeView<P> for T
where
    T: Value<P> + Diff,
{
    type Product = T::Memo;

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

impl<P> AttributeView<P> for String
where
    P: for<'a> Property<&'a str>,
{
    type Product = String;

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

// pub struct AttributeNode<A, V> {
//     constructor: A,
//     value: V,
// }

// macro_rules! def_attr {
//     ($($name:ident,)*) => {
//         $(
//             #[doc = concat!("The `", stringify!($name) ,"` attribute constructor")]
//             pub fn $name<V>(value: V) -> AttributeNode<impl Fn() -> Node, V> {
//                 AttributeNode::new(util::$name, value)
//             }
//         )*
//     };
// }

// def_attr! {
//     href,
//     style,
//     value,
// }

// impl<A, T> AttributeNode<A, T> {
//     pub const fn new(constructor: A, value: T) -> Self {
//         AttributeNode { constructor, value }
//     }
// }

// impl<A, T> View for AttributeNode<A, T>
// where
//     A: Fn() -> Node,
//     T: IntoText + Diff + Copy,
// {
//     type Product = AttributeNodeProduct<T::State>;

//     fn build(self) -> Self::Product {
//         let el = Element::new((self.constructor)());

//         self.value.set_attr(&el);
//         let state = self.value.init();

//         AttributeNodeProduct { state, el }
//     }

//     fn update(self, p: &mut Self::Product) {
//         if self.value.update(&mut p.state) {
//             self.value.set_attr(&p.el);
//         }
//     }
// }

// pub trait Attribute {
//     type Abi: IntoWasmAbi;
//     type Product: 'static;

//     fn js(&self) -> Self::Abi;

//     fn build(self) -> Self::Product;

//     fn update(self, p: &mut Self::Product, el: &JsValue);
// }

// /// A class that interacts with `classList` property of an element
// ///
// /// <https://developer.mozilla.org/en-US/docs/Web/API/Element/classList>
// #[repr(transparent)]
// pub struct Class<T>(T);

// impl From<&'static str> for Class<&'static str> {
//     fn from(class: &'static str) -> Self {
//         debug_assert!(
//             !class.chars().any(|c| c == ' '),
//             "Class name cannot contain spaces, offending class: \"{class}\""
//         );

//         Class(class)
//     }
// }

// impl From<Option<&'static str>> for Class<&'static str> {
//     fn from(class: Option<&'static str>) -> Self {
//         Class::from(class.unwrap_or_default())
//     }
// }

// #[derive(Clone, Copy)]
// pub struct OptionalClass<'a> {
//     class: &'a str,
//     on: bool,
// }

// impl<'a> OptionalClass<'a> {
//     pub const fn new(class: &'a str, on: bool) -> Self {
//         OptionalClass { class, on }
//     }

//     pub const fn no_diff(self) -> NoDiff<Self> {
//         NoDiff(self)
//     }

//     fn get(&self) -> &'a str {
//         if self.on {
//             self.class
//         } else {
//             ""
//         }
//     }
// }

// impl<'a> From<OptionalClass<'a>> for Class<OptionalClass<'a>> {
//     fn from(class: OptionalClass<'a>) -> Self {
//         Class(class)
//     }
// }

// impl<'a> From<NoDiff<OptionalClass<'a>>> for Class<NoDiff<OptionalClass<'a>>> {
//     fn from(class: NoDiff<OptionalClass<'a>>) -> Self {
//         Class(class)
//     }
// }

// impl Attribute for Class<&'static str> {
//     type Abi = &'static str;
//     type Product = &'static str;

//     fn js(&self) -> Self::Abi {
//         self.0
//     }

//     fn build(self) -> Self::Product {
//         self.0
//     }

//     fn update(self, p: &mut Self::Product, js: &JsValue) {
//         match (self.0, *p) {
//             (new, old) if new == old => return,
//             (new, "") => util::add_class(js, new),
//             ("", old) => util::remove_class(js, old),
//             (new, old) => util::replace_class(js, old, new),
//         }
//         *p = self.0;
//     }
// }

// impl<'a> Attribute for Class<NoDiff<OptionalClass<'a>>> {
//     type Abi = &'a str;
//     type Product = bool;

//     fn js(&self) -> Self::Abi {
//         self.0.get()
//     }

//     fn build(self) -> Self::Product {
//         self.0.on
//     }

//     fn update(self, p: &mut Self::Product, js: &JsValue) {
//         match (self.0.on, *p) {
//             (true, true) | (false, false) => return,
//             (true, false) => util::add_class(js, self.0.class),
//             (false, true) => util::remove_class(js, self.0.class),
//         }
//         *p = self.0.on;
//     }
// }

// pub struct AttributeNodeProduct<V> {
//     state: V,
//     el: Element,
// }

// impl<V: 'static> Mountable for AttributeNodeProduct<V> {
//     type Js = JsValue;

//     fn el(&self) -> &Element {
//         &self.el
//     }
// }
