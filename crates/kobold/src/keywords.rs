// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Keyword handles for `{ ... }` expressions in the [`view!`](crate::view) macro.

use crate::View;
use crate::diff::{Eager, Static};
use crate::list::{Bounded, List};

/// `{ for ... }`: turn an [`IntoIterator`] type into a [`View`].
///
/// ```
/// # use kobold::prelude::*;
/// view! {
///     <h1>"Integers 1 to 10:"</h1>
///     <ul>
///     { for (1..=10).map(|n| view! { <li>{ n } }) }
///     </ul>
/// }
/// # ;
/// ```
pub const fn r#for<T>(iterator: T) -> List<T>
where
    T: IntoIterator,
    T::Item: View,
{
    List::new(iterator)
}

/// `{ for<N> ... }`: turn an [`IntoIterator`] type into a [`View`],
/// bounded to max length of `N`.
///
/// This should be used only for small values of `N`.
///
/// # Performance
///
/// The main advantage in using `for<N>` over regular `for` is that the
/// bounded variant of a [`List`] doesn't need to allocate as the max size is fixed
/// and known at compile time.
///
/// ```
/// # use kobold::prelude::*;
/// view! {
///     <h1>"Integers 1 to 10:"</h1>
///     <ul>
///     { for<10> (1..=10).map(|n| view! { <li>{ n } }) }
///     </ul>
/// }
/// # ;
/// ```
pub const fn for_bounded<T, const N: usize>(iterator: T) -> List<T, Bounded<N>> {
    List::new_bounded(iterator)
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
