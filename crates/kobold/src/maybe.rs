// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The [`Maybe`](Maybe) trait and its implementations

/// Undefined component parameter. If you've encountered this type it usually means
/// you've failed to set a required component parameter.
pub struct Undefined;

/// Helper trait for handling optional parameters in components.
pub trait Maybe<T> {
    fn maybe_or(self, or: impl FnOnce() -> T) -> T;
}

impl<T> Maybe<T> for T
where
    T: Default,
{
    /// This implementation is a no-op that always returns `self`
    fn maybe_or(self, _: impl FnOnce() -> T) -> T {
        self
    }
}

impl<T> Maybe<T> for Option<T> {
    fn maybe_or(self, or: impl FnOnce() -> T) -> T {
        self.unwrap_or_else(or)
    }
}

impl<T> Maybe<Option<T>> for T
where
    T: Default,
{
    fn maybe_or(self, _: impl FnOnce() -> Option<T>) -> Option<T> {
        Some(self)
    }
}

impl<T> Maybe<T> for Undefined {
    /// This implementation is a no-op that always returns the result of
    /// the `or` closure
    fn maybe_or(self, or: impl FnOnce() -> T) -> T {
        or()
    }
}
