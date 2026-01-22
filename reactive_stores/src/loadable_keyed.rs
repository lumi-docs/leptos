//! Loadable keyed store fields.
//!
//! Combines loading state with keyed reactivity for collections.
//! This allows you to:
//! - Track loading/error state for async collection fetches
//! - Get keyed iteration when data is ready (Ready contains KeyedSubfield)
//! - Maintain stable keys across re-renders

use crate::{
    keyed::KeyedSubfield,
    loadable::Loadable,
    path::StorePathSegment,
    store_field::StoreField,
    Subfield,
};
use reactive_graph::traits::{DefinedAt, IsDisposed, Track};
use std::{
    error::Error,
    fmt::Debug,
    hash::Hash,
    ops::IndexMut,
    panic::Location,
};

/// A loadable field with keyed collection support.
///
/// This type wraps a `Loadable<T, E>` and provides:
/// - State tracking: `NotStarted`, `Loading`, `Ready`, `Failed`
/// - Keyed iteration when data is ready (Ready contains KeyedSubfield)
///
/// The `.state()` method returns `Loadable<KeyedSubfield<...>, E>`:
/// - `NotStarted` - Loading hasn't begun
/// - `Loading` - Currently fetching
/// - `Ready(keyed)` - Contains a `KeyedSubfield` for iteration with `<For>`
/// - `Failed(e)` - Loading failed with error
///
/// # Type Parameters
///
/// - `Inner` - The inner store field type
/// - `Prev` - The parent struct type
/// - `K` - The key type for keyed access
/// - `T` - The collection type (e.g., `Vec<Item>`)
/// - `E` - The error type (must implement `std::error::Error`)
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Store)]
/// struct State {
///     #[store(loadable(ApiError), key: IdType<Item> = |item| item.id)]
///     items: Vec<Item>,
/// }
///
/// // Usage:
/// state.items().set_loading();
/// state.items().set_ready(fetched_items);
///
/// // Pattern matching - Ready contains KeyedSubfield
/// match state.items().state() {
///     Loadable::NotStarted => view! { "Not loaded" },
///     Loadable::Loading => view! { <Spinner /> },
///     Loadable::Ready(items) => view! {
///         // items is a KeyedSubfield - passable to <For>
///         <For
///             each=move || items
///             key=|item| item.id().get()
///             children=|item| view! { <ItemRow item=item /> }
///         />
///     },
///     Loadable::Failed(err) => view! { "Error: " {err.to_string()} },
/// }
/// ```
#[derive(Clone, Copy)]
pub struct LoadableKeyedSubfield<Inner, Prev, K, T, E>
where
    Inner: StoreField<Value = Prev>,
    for<'a> &'a T: IntoIterator,
    T: IndexMut<usize>,
    K: Hash + PartialEq + Eq,
    E: Error,
{
    #[cfg(any(debug_assertions, leptos_debuginfo))]
    defined_at: &'static Location<'static>,
    /// The inner Subfield wrapping Loadable<T, E>
    inner: Subfield<Inner, Prev, Loadable<T, E>>,
    /// Path segment for keyed access (used when constructing KeyedSubfield)
    path_segment: StorePathSegment,
    /// Key extraction function
    key_fn: fn(<&T as IntoIterator>::Item) -> K,
}

impl<Inner, Prev, K, T, E> LoadableKeyedSubfield<Inner, Prev, K, T, E>
where
    Inner: StoreField<Value = Prev>,
    Prev: 'static,
    for<'a> &'a T: IntoIterator,
    T: IndexMut<usize>,
    K: Debug + Send + Sync + Hash + PartialEq + Eq + Clone + 'static,
    E: Error + Clone + 'static,
{
    /// Creates a new loadable keyed subfield.
    ///
    /// # Arguments
    ///
    /// * `inner` - The Subfield wrapping `Loadable<T, E>`
    /// * `path_segment` - The path segment for this field
    /// * `key_fn` - Function to extract keys from collection items
    #[track_caller]
    pub fn new(
        inner: Subfield<Inner, Prev, Loadable<T, E>>,
        path_segment: StorePathSegment,
        key_fn: fn(<&T as IntoIterator>::Item) -> K,
    ) -> Self {
        Self {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: Location::caller(),
            inner,
            path_segment,
            key_fn,
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

    /// Set to `Ready` with the given collection.
    ///
    /// The collection becomes available via `.state()` which returns
    /// `Loadable::Ready(KeyedSubfield<...>)`.
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
    /// Returns `Loadable<KeyedSubfield<...>, E>`:
    /// - `Ready(keyed)` contains a `KeyedSubfield` for iteration
    /// - Other variants contain no data
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match state.items().state() {
    ///     Loadable::NotStarted => view! { "Click to load" },
    ///     Loadable::Loading => view! { <Spinner /> },
    ///     Loadable::Ready(items) => view! {
    ///         // items is KeyedSubfield - passable to <For>
    ///         <For
    ///             each=move || items
    ///             key=|item| item.id().get()
    ///             children=|item| view! { <ItemRow item=item /> }
    ///         />
    ///     },
    ///     Loadable::Failed(ref err) => view! { "Error: " {err.to_string()} },
    /// }
    /// ```
    pub fn state(
        &self,
    ) -> Loadable<
        KeyedSubfield<Subfield<Inner, Prev, Loadable<T, E>>, Loadable<T, E>, K, T>,
        E,
    >
    where
        Inner: Clone,
        T: 'static,
    {
        self.inner.track_field();
        self.state_untracked()
    }

    /// Get the current loading state without tracking.
    pub fn state_untracked(
        &self,
    ) -> Loadable<
        KeyedSubfield<Subfield<Inner, Prev, Loadable<T, E>>, Loadable<T, E>, K, T>,
        E,
    >
    where
        Inner: Clone,
        T: 'static,
    {
        match self.inner.reader().as_deref() {
            Some(Loadable::NotStarted) | None => Loadable::NotStarted,
            Some(Loadable::Loading) => Loadable::Loading,
            Some(Loadable::Ready(_)) => {
                // Create a KeyedSubfield that accesses the T inside Loadable<T, E>
                let keyed = KeyedSubfield::new(
                    self.inner.clone(),
                    self.path_segment,
                    self.key_fn,
                    // Read accessor: unwrap Ready to get &T
                    |loadable: &Loadable<T, E>| match loadable {
                        Loadable::Ready(t) => t,
                        _ => panic!(
                            "LoadableKeyedSubfield: attempted to access data when not Ready"
                        ),
                    },
                    // Write accessor: unwrap Ready to get &mut T
                    |loadable: &mut Loadable<T, E>| match loadable {
                        Loadable::Ready(t) => t,
                        _ => panic!(
                            "LoadableKeyedSubfield: attempted to mutate data when not Ready"
                        ),
                    },
                );
                Loadable::Ready(keyed)
            }
            Some(Loadable::Failed(e)) => Loadable::Failed(e.clone()),
        }
    }

    /// Update the collection if currently ready, otherwise do nothing.
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

impl<Inner, Prev, K, T, E> DefinedAt
    for LoadableKeyedSubfield<Inner, Prev, K, T, E>
where
    Inner: StoreField<Value = Prev>,
    for<'a> &'a T: IntoIterator,
    T: IndexMut<usize>,
    K: Hash + PartialEq + Eq,
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

impl<Inner, Prev, K, T, E> IsDisposed
    for LoadableKeyedSubfield<Inner, Prev, K, T, E>
where
    Inner: StoreField<Value = Prev> + IsDisposed,
    for<'a> &'a T: IntoIterator,
    T: IndexMut<usize>,
    K: Hash + PartialEq + Eq,
    E: Error,
{
    fn is_disposed(&self) -> bool {
        self.inner.is_disposed()
    }
}

impl<Inner, Prev, K, T, E> Track
    for LoadableKeyedSubfield<Inner, Prev, K, T, E>
where
    Inner: StoreField<Value = Prev> + Track + Clone + 'static,
    Prev: 'static,
    for<'a> &'a T: IntoIterator,
    T: IndexMut<usize> + 'static,
    K: Debug + Send + Sync + Hash + PartialEq + Eq + Clone + 'static,
    E: Error + 'static,
{
    fn track(&self) {
        self.inner.track();
    }
}

impl<Inner, Prev, K, T, E> std::fmt::Debug
    for LoadableKeyedSubfield<Inner, Prev, K, T, E>
where
    Inner: StoreField<Value = Prev>,
    Prev: 'static,
    for<'a> &'a T: IntoIterator,
    T: IndexMut<usize>,
    K: Debug + Send + Sync + Hash + PartialEq + Eq + Clone + 'static,
    E: Error + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadableKeyedSubfield")
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
