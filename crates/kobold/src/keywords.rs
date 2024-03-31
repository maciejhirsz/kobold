// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Keyword handles for `{ ... }` expressions in the [`view!`](crate::view) macro.

use crate::diff::{Eager, Ref, Static};
use crate::list::List;
use crate::View;

/// `{ for ... }`: turn an [`IntoIterator`] type into a [`View`].
///
/// ```
/// # use kobold::prelude::*;
/// view! {
///     <h1>"Integers 1 to 10:"</h1>
///     <ul>
///     { for (1..=10).map(|n| view! { <li>{ n }</li> }) }
///     </ul>
/// }
/// # ;
/// ```
pub const fn r#for<T>(iterator: T) -> List<T>
where
    T: IntoIterator,
    T::Item: View,
{
    List(iterator)
}

/// `{ ref ... }`: diff this value by its reference address.
///
/// For strings this is both faster and more memory efficient (no allocations necessary),
/// however it might fail to update if underlying memory has been mutated in place without
/// re-allocations.
/// ```
/// use kobold::prelude::*;
///
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// #[component]
/// fn user_row(user: &User) -> impl View + '_ {
///     view! {
///         <tr>
///             // If `name` and `email` are always sent to the UI as
///             // newly allocated `String`s, it's both safe and faster
///             // to diff them by reference than value.
///             <td>{ ref user.name }</td>
///             <td>{ ref user.email }</td>
///         </tr>
///     }
/// }
/// # fn main() {}
/// ```
pub const fn r#ref(value: &str) -> &Ref<str> {
    unsafe { &*(value as *const _ as *const Ref<str>) }
}

/// `{ use ... }`: disable diffing for `T` and apply its value to the DOM on every render.
///
/// This is usually not advised, but can be useful when combined with [`fence`](crate::diff::fence).
pub const fn r#use<T>(value: T) -> Eager<T> {
    Eager(value)
}

/// `{ static ... }` disable diffing for `T` and never update its value in the DOM after the initial render.
pub const fn r#static<T>(value: T) -> Static<T> {
    Static(value)
}

/// `{ do ... }` is an alias for [`{ event!(...) }`](../macro.event.html)
pub use crate::event as r#do;
