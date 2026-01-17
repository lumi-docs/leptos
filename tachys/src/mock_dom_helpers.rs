//! Mock DOM helper functions for mock_dom mode.
//!
//! These provide stub implementations of browser-specific DOM functions.

use crate::renderer::mock_dom::events::{FromEventTarget, JsValue};
use crate::renderer::mock_dom::{Document, Element, MockDom};

/// Returns a mock window.
/// Returns Some to match the real web_sys::window() signature.
pub fn window() -> Option<MockWindow> {
    Some(MockWindow)
}

/// Returns a mock document.
/// Returns Some to match the pattern for optional browser APIs.
pub fn document() -> Option<MockDocument> {
    Some(MockDocument)
}

/// Returns a mock body element.
pub fn body() -> Element {
    MockDom::create_element("body", None)
}

/// Mock window type.
#[derive(Clone, Debug)]
pub struct MockWindow;

impl MockWindow {
    /// Returns a mock location.
    pub fn location(&self) -> MockLocation {
        MockLocation
    }

    /// Returns a mock document.
    pub fn document(&self) -> Option<MockDocument> {
        Some(MockDocument)
    }
}

/// Mock document type.
#[derive(Clone, Debug)]
pub struct MockDocument;

impl MockDocument {
    /// Returns a mock head element.
    pub fn head(&self) -> Option<Element> {
        Some(MockDom::create_element("head", None))
    }

    /// Returns a mock body element.
    pub fn body(&self) -> Option<Element> {
        Some(MockDom::create_element("body", None))
    }

    /// Returns a mock document element.
    pub fn document_element(&self) -> Option<Element> {
        Some(MockDom::create_element("html", None))
    }

    /// Gets element by ID (always returns None in mock mode).
    pub fn get_element_by_id(&self, _id: &str) -> Option<Element> {
        None
    }

    /// Query for an element matching the given CSS selector.
    ///
    /// Returns the first element that matches the selector.
    /// Mirrors the web_sys::Document::query_selector API.
    pub fn query_selector(
        &self,
        selector: &str,
    ) -> Result<Option<Element>, JsValue> {
        Ok(Document::query_selector(selector))
    }
}

/// Mock location type.
#[derive(Clone, Debug)]
pub struct MockLocation;

#[allow(clippy::result_unit_err)]
impl MockLocation {
    /// Returns empty hash.
    pub fn hash(&self) -> Result<String, ()> {
        Ok(String::new())
    }

    /// Returns root pathname.
    pub fn pathname(&self) -> Result<String, ()> {
        Ok(String::from("/"))
    }

    /// Returns localhost origin.
    pub fn origin(&self) -> Result<String, ()> {
        Ok(String::from("http://localhost"))
    }

    /// Returns localhost href.
    pub fn href(&self) -> Result<String, ()> {
        Ok(String::from("http://localhost/"))
    }

    /// Sets href (no-op).
    pub fn set_href(&self, _href: &str) -> Result<(), ()> {
        Ok(())
    }
}

/// Helper function to extract event target.
/// Returns the event target cast to the specified element type.
pub fn event_target<T>(event: &crate::renderer::mock_dom::events::Event) -> T
where
    T: crate::renderer::mock_dom::events::FromEventTarget,
{
    event
        .target()
        .and_then(|t| T::from_event_target(t).ok())
        .expect("event_target: event had no target or wrong element type")
}

/// Helper function to extract event target value (stub).
pub fn event_target_value<T>(_event: &T) -> String {
    String::new()
}

/// Helper function to extract event target checked.
/// Returns the checked state of an input element.
pub fn event_target_checked(
    ev: &crate::renderer::mock_dom::events::Event,
) -> bool {
    use crate::renderer::mock_dom::events::HtmlInputElement;
    ev.target()
        .and_then(|t| HtmlInputElement::from_event_target(t).ok())
        .map(|input: HtmlInputElement| input.checked())
        .unwrap_or(false)
}
