//! Mock location provider for testing without browser APIs.
//!
//! This module provides `MockUrl`, a `LocationProvider` implementation that works
//! entirely in memory without any browser dependencies (window, history, etc.).

use super::{LocationChange, LocationProvider, Url};
use crate::params::ParamsMap;
use core::fmt;
use leptos::prelude::*;
use reactive_graph::{
    signal::{ArcRwSignal, ReadSignal, RwSignal},
    traits::{ReadUntracked, Set},
};
use std::borrow::Cow;

/// A mock location provider for testing.
///
/// Unlike `BrowserUrl`, this doesn't interact with browser APIs at all.
/// All URL state is stored in memory and can be manipulated programmatically.
#[derive(Clone)]
pub struct MockUrl {
    url: ArcRwSignal<Url>,
    is_back: RwSignal<bool>,
    path_stack: StoredValue<Vec<Url>>,
}

impl fmt::Debug for MockUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockUrl")
            .field("url", &self.url.read_untracked())
            .finish_non_exhaustive()
    }
}

impl Default for MockUrl {
    fn default() -> Self {
        Self::new().expect("MockUrl::new() should not fail")
    }
}

impl MockUrl {
    /// Creates a new MockUrl at a specific path.
    pub fn with_path(path: &str) -> Self {
        let url = Self::parse(path).unwrap_or_else(|_| Url {
            origin: "http://localhost".to_string(),
            path: path.to_string(),
            search: String::new(),
            search_params: ParamsMap::new(),
            hash: String::new(),
        });
        Self {
            url: ArcRwSignal::new(url.clone()),
            is_back: RwSignal::new(false),
            path_stack: StoredValue::new(vec![url]),
        }
    }

    /// Navigate to a new URL (for testing).
    pub fn navigate_to(&self, path: &str) {
        if let Ok(url) = Self::parse(path) {
            self.path_stack
                .update_value(|stack| stack.push(url.clone()));
            self.url.set(url);
            self.is_back.set(false);
        }
    }

    /// Simulate a back navigation (for testing).
    pub fn go_back(&self) {
        self.path_stack.update_value(|stack| {
            if stack.len() > 1 {
                stack.pop();
                if let Some(prev) = stack.last().cloned() {
                    self.is_back.set(true);
                    self.url.set(prev);
                }
            }
        });
    }
}

#[allow(clippy::result_unit_err)]
impl LocationProvider for MockUrl {
    type Error = ();

    fn new() -> Result<Self, Self::Error> {
        let url = Url {
            origin: "http://localhost".to_string(),
            path: "/".to_string(),
            search: String::new(),
            search_params: ParamsMap::new(),
            hash: String::new(),
        };
        Ok(Self {
            url: ArcRwSignal::new(url.clone()),
            is_back: RwSignal::new(false),
            path_stack: StoredValue::new(vec![url]),
        })
    }

    fn as_url(&self) -> &ArcRwSignal<Url> {
        &self.url
    }

    fn current() -> Result<Url, Self::Error> {
        // In mock mode, there's no global "current" URL - return a default
        Ok(Url {
            origin: "http://localhost".to_string(),
            path: "/".to_string(),
            search: String::new(),
            search_params: ParamsMap::new(),
            hash: String::new(),
        })
    }

    fn init(&self, _base: Option<Cow<'static, str>>) {
        // No-op in mock mode - no event listeners to set up
    }

    fn ready_to_complete(&self) {
        // No-op - no pending navigation tracking needed in mock mode
    }

    fn complete_navigation(&self, loc: &LocationChange) {
        if let Ok(url) = Self::parse(&loc.value) {
            if !loc.replace {
                self.path_stack
                    .update_value(|stack| stack.push(url.clone()));
            }
            self.url.set(url);
            self.is_back.set(false);
        }
    }

    fn parse_with_base(url_str: &str, base: &str) -> Result<Url, Self::Error> {
        // Use the `url` crate for parsing
        let base_url = url::Url::parse(base).map_err(|_| ())?;
        let parsed = base_url.join(url_str).map_err(|_| ())?;

        let search_params: ParamsMap = parsed
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(Url {
            origin: parsed.origin().ascii_serialization(),
            path: parsed.path().to_string(),
            search: parsed.query().unwrap_or("").to_string(),
            search_params,
            hash: parsed
                .fragment()
                .map(|s| format!("#{s}"))
                .unwrap_or_default(),
        })
    }

    fn redirect(_loc: &str) {
        // No-op in mock mode - can't redirect without a browser
        // In tests, you would check the URL signal instead
    }

    fn is_back(&self) -> ReadSignal<bool> {
        self.is_back.read_only()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_url_new() {
        let mock = MockUrl::new().unwrap();
        let url = mock.url.read_untracked();
        assert_eq!(url.path(), "/");
        assert_eq!(url.origin(), "http://localhost");
    }

    #[test]
    fn test_mock_url_with_path() {
        let mock = MockUrl::with_path("/dashboard/settings");
        let url = mock.url.read_untracked();
        assert_eq!(url.path(), "/dashboard/settings");
    }

    #[test]
    fn test_parse_with_base() {
        let url = MockUrl::parse_with_base(
            "/test?foo=bar#hash",
            "http://example.com",
        )
        .unwrap();
        assert_eq!(url.path(), "/test");
        assert_eq!(url.search(), "foo=bar");
        assert_eq!(url.hash(), "#hash");
        assert_eq!(url.search_params().get_str("foo"), Some("bar"));
    }

    #[test]
    fn test_parse_absolute_url() {
        let url = MockUrl::parse_with_base(
            "http://other.com/path",
            "http://example.com",
        )
        .unwrap();
        assert_eq!(url.origin(), "http://other.com");
        assert_eq!(url.path(), "/path");
    }
}
