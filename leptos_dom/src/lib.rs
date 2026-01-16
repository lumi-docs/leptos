#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! DOM helpers for Leptos.

#[cfg(not(feature = "mock_dom"))]
pub mod helpers;

#[cfg(feature = "mock_dom")]
#[path = "mock_helpers.rs"]
pub mod helpers;

#[doc(hidden)]
pub mod macro_helpers;

/// Utilities for simple isomorphic logging to the console or terminal.
pub mod logging;
