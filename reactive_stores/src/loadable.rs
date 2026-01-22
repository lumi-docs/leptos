//! Loadable (async loading state) store fields.
//!
//! A loadable field wraps data that may be in various loading states:
//! - `NotStarted` - Loading hasn't begun
//! - `Loading` - Currently fetching data
//! - `Ready(T)` - Data is available
//! - `Failed(E)` - Loading failed with an error
//!
//! Unlike regular [`Subfield`](crate::Subfield)s, loadable fields:
//! - Don't expose raw `.get()/.set()` on the wrapper
//! - Provide controlled methods: `.try_get()`, `.set_ready()`, etc.
//! - Support pattern matching via `.with()` for reactive rendering

use crate::{store_field::StoreField, Subfield};
use reactive_graph::traits::{DefinedAt, IsDisposed, Track};
use std::{error::Error, panic::Location};

/// Loading state for async data.
///
/// Generic over:
/// - `T` - The data type when ready
/// - `E` - The error type when failed (must implement `std::error::Error`)
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Store)]
/// struct State {
///     #[store(loadable)]
///     profile: Loadable<UserProfile, FetchError>,
/// }
///
/// // Usage:
/// state.profile().set_loading();
/// state.profile().set_ready(fetched_profile);
/// state.profile().with(
///     || view! { "Not loaded" },
///     || view! { "Loading..." },
///     |profile| view! { <Profile data=profile.clone() /> },
///     |err| view! { "Error: " {err} },
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loadable<T, E: Error> {
    /// Loading has not started yet.
    NotStarted,
    /// Currently loading data.
    Loading,
    /// Data is ready and available.
    Ready(T),
    /// Loading failed with an error.
    Failed(E),
}

impl<T, E: Error> Default for Loadable<T, E> {
    fn default() -> Self {
        Self::NotStarted
    }
}

impl<T, E: Error> Loadable<T, E> {
    /// Returns `true` if loading has not started.
    pub fn is_not_started(&self) -> bool {
        matches!(self, Self::NotStarted)
    }

    /// Returns `true` if currently loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns `true` if data is ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Returns `true` if loading failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// Converts from `&Loadable<T, E>` to `Loadable<&T, &E>`.
    pub fn as_ref(&self) -> Loadable<&T, &E> {
        match self {
            Loadable::NotStarted => Loadable::NotStarted,
            Loadable::Loading => Loadable::Loading,
            Loadable::Ready(v) => Loadable::Ready(v),
            Loadable::Failed(e) => Loadable::Failed(e),
        }
    }

    /// Returns the contained value if `Ready`, or `None` otherwise.
    pub fn as_ready(&self) -> Option<&T> {
        match self {
            Loadable::Ready(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the error if `Failed`, or `None` otherwise.
    pub fn as_failed(&self) -> Option<&E> {
        match self {
            Loadable::Failed(e) => Some(e),
            _ => None,
        }
    }

    /// Maps a `Loadable<T, E>` to `Loadable<U, E>` by applying a function to the ready value.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Loadable<U, E> {
        match self {
            Loadable::NotStarted => Loadable::NotStarted,
            Loadable::Loading => Loadable::Loading,
            Loadable::Ready(v) => Loadable::Ready(f(v)),
            Loadable::Failed(e) => Loadable::Failed(e),
        }
    }

    /// Maps a `Loadable<T, E>` to `Loadable<T, F>` by applying a function to the error.
    pub fn map_err<F: Error, G: FnOnce(E) -> F>(self, f: G) -> Loadable<T, F> {
        match self {
            Loadable::NotStarted => Loadable::NotStarted,
            Loadable::Loading => Loadable::Loading,
            Loadable::Ready(v) => Loadable::Ready(v),
            Loadable::Failed(e) => Loadable::Failed(f(e)),
        }
    }
}

/// A controlled accessor for loadable fields.
///
/// This type does NOT expose raw `.get()/.set()` on the `Loadable` wrapper.
/// Instead, it provides controlled methods for state transitions:
/// - `.try_get()` - Get the inner value if ready
/// - `.set_loading()` - Transition to loading state
/// - `.set_ready(value)` - Set the ready value
/// - `.set_failed(error)` - Set the error state
/// - `.with(...)` - Pattern match on state with closures
///
/// # Type Parameters
///
/// - `Inner` - The inner store field type
/// - `Prev` - The parent struct type
/// - `T` - The data type when ready
/// - `E` - The error type when failed
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Store)]
/// struct State {
///     #[store(loadable)]
///     profile: Loadable<UserProfile, FetchError>,
/// }
///
/// // The accessor returns LoadableSubfield, not Subfield
/// let profile = state.profile();
///
/// // Controlled state transitions
/// profile.set_loading();
/// profile.set_ready(UserProfile { name: "Alice".into() });
///
/// // Pattern matching for reactive rendering
/// profile.with(
///     || view! { "Click to load" },
///     || view! { <Spinner /> },
///     |p| view! { <ProfileCard profile=p.clone() /> },
///     |e| view! { <Error message=e.to_string() /> },
/// )
/// ```
#[derive(Clone, Copy)]
pub struct LoadableSubfield<Inner, Prev, T, E>
where
    Inner: StoreField<Value = Prev>,
    E: Error,
{
    #[cfg(any(debug_assertions, leptos_debuginfo))]
    defined_at: &'static Location<'static>,
    inner: Subfield<Inner, Prev, Loadable<T, E>>,
}

impl<Inner, Prev, T, E> LoadableSubfield<Inner, Prev, T, E>
where
    Inner: StoreField<Value = Prev>,
    Prev: 'static,
    T: Clone + 'static,
    E: Error + Clone + 'static,
{
    /// Creates a new loadable subfield wrapping a Subfield.
    #[track_caller]
    pub fn new(inner: Subfield<Inner, Prev, Loadable<T, E>>) -> Self {
        Self {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: Location::caller(),
            inner,
        }
    }

    /// Returns `true` if loading has not started.
    pub fn is_not_started(&self) -> bool {
        self.inner
            .reader()
            .map(|r| r.is_not_started())
            .unwrap_or(true)
    }

    /// Returns `true` if currently loading.
    pub fn is_loading(&self) -> bool {
        self.inner
            .reader()
            .map(|r| r.is_loading())
            .unwrap_or(false)
    }

    /// Returns `true` if data is ready.
    pub fn is_ready(&self) -> bool {
        self.inner.reader().map(|r| r.is_ready()).unwrap_or(false)
    }

    /// Returns `true` if loading failed.
    pub fn is_failed(&self) -> bool {
        self.inner.reader().map(|r| r.is_failed()).unwrap_or(false)
    }

    /// Try to get the inner value (returns `None` if not `Ready`).
    ///
    /// This is reactive - it will track the field.
    pub fn try_get(&self) -> Option<T> {
        self.inner.track_field();
        self.inner.reader().and_then(|r| match &*r {
            Loadable::Ready(v) => Some(v.clone()),
            _ => None,
        })
    }

    /// Try to get the inner value without tracking.
    pub fn try_get_untracked(&self) -> Option<T> {
        self.inner.reader().and_then(|r| match &*r {
            Loadable::Ready(v) => Some(v.clone()),
            _ => None,
        })
    }

    /// Get the error if failed.
    ///
    /// This is reactive - it will track the field.
    pub fn error(&self) -> Option<E> {
        self.inner.track_field();
        self.inner.reader().and_then(|r| match &*r {
            Loadable::Failed(e) => Some(e.clone()),
            _ => None,
        })
    }

    /// Get the error if failed, without tracking.
    pub fn error_untracked(&self) -> Option<E> {
        self.inner.reader().and_then(|r| match &*r {
            Loadable::Failed(e) => Some(e.clone()),
            _ => None,
        })
    }

    /// Set to `Loading` state.
    pub fn set_loading(&self) {
        if let Some(mut w) = self.inner.writer() {
            *w = Loadable::Loading;
        }
    }

    /// Set to `Ready` with the given value.
    pub fn set_ready(&self, value: T) {
        if let Some(mut w) = self.inner.writer() {
            *w = Loadable::Ready(value);
        }
    }

    /// Set to `Failed` with the given error.
    pub fn set_failed(&self, error: E) {
        if let Some(mut w) = self.inner.writer() {
            *w = Loadable::Failed(error);
        }
    }

    /// Reset to `NotStarted` state.
    pub fn reset(&self) {
        if let Some(mut w) = self.inner.writer() {
            *w = Loadable::NotStarted;
        }
    }

    /// Track this field for reactivity.
    pub fn track(&self) {
        self.inner.track_field();
    }

    /// Get the current loading state for pattern matching.
    ///
    /// This is reactive - it tracks the field.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match state.profile().state() {
    ///     Loadable::NotStarted => view! { "Not loaded yet" },
    ///     Loadable::Loading => view! { <Spinner /> },
    ///     Loadable::Ready(ref profile) => view! { <ProfileCard data=profile.clone() /> },
    ///     Loadable::Failed(ref err) => view! { "Error: " {err.to_string()} },
    /// }
    /// ```
    pub fn state(&self) -> Loadable<T, E> {
        self.inner.track_field();
        self.inner
            .reader()
            .map(|r| (*r).clone())
            .unwrap_or(Loadable::NotStarted)
    }

    /// Get the current loading state without tracking.
    pub fn state_untracked(&self) -> Loadable<T, E> {
        self.inner
            .reader()
            .map(|r| (*r).clone())
            .unwrap_or(Loadable::NotStarted)
    }

    /// Update the ready value if currently ready, otherwise do nothing.
    ///
    /// Returns `true` if the update was applied.
    pub fn update_if_ready(&self, f: impl FnOnce(&mut T)) -> bool {
        if let Some(mut w) = self.inner.writer() {
            if let Loadable::Ready(ref mut v) = *w {
                f(v);
                return true;
            }
        }
        false
    }
}

impl<Inner, Prev, T, E> DefinedAt for LoadableSubfield<Inner, Prev, T, E>
where
    Inner: StoreField<Value = Prev>,
    E: Error,
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

impl<Inner, Prev, T, E> IsDisposed for LoadableSubfield<Inner, Prev, T, E>
where
    Inner: StoreField<Value = Prev> + IsDisposed,
    E: Error,
{
    fn is_disposed(&self) -> bool {
        self.inner.is_disposed()
    }
}

impl<Inner, Prev, T, E> Track for LoadableSubfield<Inner, Prev, T, E>
where
    Inner: StoreField<Value = Prev> + Track + 'static,
    Prev: 'static,
    T: 'static,
    E: Error + 'static,
{
    fn track(&self) {
        self.inner.track();
    }
}

impl<Inner, Prev, T, E> std::fmt::Debug for LoadableSubfield<Inner, Prev, T, E>
where
    Inner: StoreField<Value = Prev>,
    Prev: 'static,
    T: Clone + std::fmt::Debug + 'static,
    E: Error + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadableSubfield")
            .field(
                "state",
                &self.inner.reader().map(|r| match &*r {
                    Loadable::NotStarted => "NotStarted",
                    Loadable::Loading => "Loading",
                    Loadable::Ready(_) => "Ready(...)",
                    Loadable::Failed(_) => "Failed(...)",
                }),
            )
            .finish()
    }
}
