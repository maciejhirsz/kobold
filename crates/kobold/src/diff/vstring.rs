// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::diff::Diff;
use std::ops::{Deref, DerefMut};

/// Versioned string.
///
/// This type acts as a drop-in replacement for a regular owned [`String`](String). Internally its
/// memory layout is identical, except in addition to pointer, length, and capacity, `VString` also
/// tracks a _version_ integer that's automatically incremented on any mutable access.
///
/// Using a `&VString` reference as a [`View`](crate::View) is more efficient than `&String` (or `&str`), as it performs
/// no allocations and only needs to compare the version and pointer address to reliably determine if
/// the contents of the string have changed.
///
/// For best performance `VString` should always be used in place of a `String` when using the
/// [`ref`](crate::keywords::ref) keyword is not viable.
///
/// You may add the `serde` feature to the `kobold` crate to add support for the `serde::Serialize` and
/// `serde::Deserialize` traits.
pub struct VString {
    inner: String,
    ver: usize,
}

impl VString {
    /// Creates a new empty `VString`.
    pub fn new() -> VString {
        VString {
            inner: String::new(),
            ver: 0,
        }
    }

    /// Creates a new empty `VString` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> VString {
        VString {
            inner: String::with_capacity(capacity),
            ver: 0,
        }
    }

    /// Get the underlying `String` from this `VString`.
    pub fn into_inner(self) -> String {
        self.inner
    }
}

impl Diff for &'_ VString {
    type Memo = u64;

    fn into_memo(self) -> Self::Memo {
        (self.ver as u64).wrapping_shl(32) | self.inner.as_ptr() as u64
    }

    fn diff(self, memo: &mut Self::Memo) -> bool {
        let m = self.into_memo();

        if *memo != m {
            *memo = m;
            true
        } else {
            false
        }
    }
}

impl Deref for VString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for VString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ver += 1;

        &mut self.inner
    }
}

impl<S> From<S> for VString
where
    String: From<S>,
{
    fn from(value: S) -> Self {
        VString {
            inner: String::from(value),
            ver: 0,
        }
    }
}

impl Into<String> for VString {
    fn into(self) -> String {
        self.inner
    }
}

impl<S> PartialEq<S> for VString
where
    String: PartialEq<S>,
{
    fn eq(&self, other: &S) -> bool {
        self.inner.eq(other)
    }
}

impl<A> FromIterator<A> for VString
where
    String: FromIterator<A>,
{
    fn from_iter<T>(iter: T) -> VString
    where
        T: IntoIterator<Item = A>,
    {
        VString {
            inner: String::from_iter(iter),
            ver: 0,
        }
    }
}

#[cfg(feature = "serde")]
mod serde {
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::{Serialize, Serializer};

    use super::VString;

    impl Serialize for VString {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for VString {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer).map(|inner| VString { inner, ver: 0 })
        }
    }
}