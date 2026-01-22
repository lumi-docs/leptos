//! Derived (computed) store fields.
//!
//! A derived field is a read-only field backed by a [`Memo`]. Unlike regular
//! [`Subfield`](crate::Subfield)s, derived fields:
//! - Are computed from other state
//! - Have no `writer()` - they are read-only
//! - Track via the Memo, not the store path hierarchy

use crate::{
    path::{StorePath, StorePathSegment},
    store_field::StoreField,
    KeyMap, StoreFieldTrigger,
};
use reactive_graph::{
    computed::Memo,
    traits::{
        DefinedAt, Get, GetUntracked, IsDisposed, ReadUntracked, Track,
        UntrackableGuard,
    },
};
use std::{iter, marker::PhantomData, ops::Deref, panic::Location};

/// A read-only store field backed by a Memo<T>.
///
/// Unlike regular `Subfield`, this field:
/// - Is computed from other state (derived)
/// - Has no `writer()` - it's read-only
/// - Tracks via the Memo, not store path hierarchy
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Store)]
/// struct State {
///     value: i32,
///
///     #[store(derived = |store| store.value().get() * 2)]
///     doubled: i32,  // Returns DerivedField<i32>
/// }
///
/// // Usage:
/// let doubled = state.doubled().get();  // Read-only
/// state.doubled().track();  // Reactive tracking
/// // state.doubled().set(...) // Compile error! No such method
/// ```
#[derive(Clone, Copy)]
pub struct DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    #[cfg(any(debug_assertions, leptos_debuginfo))]
    defined_at: &'static Location<'static>,
    memo: Memo<T>,
}

impl<T> DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Creates a new derived field from a Memo.
    #[track_caller]
    pub fn new(memo: Memo<T>) -> Self {
        Self {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: Location::caller(),
            memo,
        }
    }

    /// Gets the current value (reactive).
    pub fn get(&self) -> T {
        self.memo.get()
    }

    /// Gets the current value (untracked).
    pub fn get_untracked(&self) -> T {
        self.memo.get_untracked()
    }
}

/// Reader for DerivedField - wraps the value.
pub struct DerivedReader<T>(T);

impl<T> Deref for DerivedReader<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Writer that can never be constructed (read-only).
///
/// This type is uninhabited - it can never be instantiated.
/// It exists only to satisfy the `StoreField::Writer` associated type
/// for read-only fields.
pub enum NeverWriter<T> {
    #[doc(hidden)]
    _Phantom(PhantomData<T>, std::convert::Infallible),
}

impl<T> Deref for NeverWriter<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            NeverWriter::_Phantom(_, inf) => match *inf {},
        }
    }
}

impl<T> std::ops::DerefMut for NeverWriter<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            NeverWriter::_Phantom(_, inf) => match *inf {},
        }
    }
}

impl<T> UntrackableGuard for NeverWriter<T> {
    fn untrack(&mut self) {
        match self {
            NeverWriter::_Phantom(_, inf) => match *inf {},
        }
    }
}

impl<T> StoreField for DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Value = T;
    type Reader = DerivedReader<T>;
    type Writer = NeverWriter<T>;

    fn get_trigger(&self, _path: StorePath) -> StoreFieldTrigger {
        // Derived fields don't participate in store trigger hierarchy
        StoreFieldTrigger::default()
    }

    fn get_trigger_unkeyed(&self, _path: StorePath) -> StoreFieldTrigger {
        StoreFieldTrigger::default()
    }

    fn path(&self) -> impl IntoIterator<Item = StorePathSegment> {
        iter::empty() // Not part of store path hierarchy
    }

    fn path_unkeyed(&self) -> impl IntoIterator<Item = StorePathSegment> {
        iter::empty()
    }

    fn track_field(&self) {
        self.memo.track(); // Track the memo, not store path
    }

    fn reader(&self) -> Option<Self::Reader> {
        Some(DerivedReader(self.memo.get()))
    }

    fn writer(&self) -> Option<Self::Writer> {
        None // Read-only - never returns a writer
    }

    fn keys(&self) -> Option<KeyMap> {
        None // Derived fields don't have keys
    }
}

impl<T> DefinedAt for DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        #[cfg(any(debug_assertions, leptos_debuginfo))]
        {
            Some(self.defined_at)
        }
        #[cfg(not(any(debug_assertions, leptos_debuginfo)))]
        {
            None
        }
    }
}

impl<T> IsDisposed for DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn is_disposed(&self) -> bool {
        // Check if the memo is disposed by trying to read from it
        self.memo.try_read_untracked().is_none()
    }
}

impl<T> Track for DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn track(&self) {
        self.memo.track();
    }
}

impl<T> ReadUntracked for DerivedField<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Value = DerivedReader<T>;

    fn try_read_untracked(&self) -> Option<Self::Value> {
        Some(DerivedReader(self.memo.get_untracked()))
    }
}

impl<T> std::fmt::Debug for DerivedField<T>
where
    T: Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedField")
            .field("value", &self.get_untracked())
            .finish()
    }
}
