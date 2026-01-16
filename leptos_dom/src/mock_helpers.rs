//! Stub implementations of DOM helper functions for mock_dom mode.
//!
//! These are no-op stubs that allow code to compile when mock_dom is enabled
//! but don't actually interact with a browser DOM.

#![allow(clippy::result_unit_err)]

use std::time::Duration;
use tachys::html::event::EventDescriptor;

/// Stub window function for mock_dom.
pub fn window() -> MockWindow {
    MockWindow
}

/// Stub document function for mock_dom.
pub fn document() -> MockDocument {
    MockDocument
}

/// Mock window type for mock_dom.
#[derive(Clone, Debug)]
pub struct MockWindow;

impl MockWindow {
    /// Returns the mock location.
    pub fn location(&self) -> MockLocation {
        MockLocation
    }
}

/// Mock document type for mock_dom.
#[derive(Clone, Debug)]
pub struct MockDocument;

impl MockDocument {
    /// Returns None (no head in mock mode).
    pub fn head(&self) -> Option<tachys::renderer::mock_dom::Element> {
        None
    }

    /// Returns None (no body in mock mode).
    pub fn body(&self) -> Option<tachys::renderer::mock_dom::Element> {
        None
    }

    /// Returns None (no document element in mock mode).
    pub fn document_element(
        &self,
    ) -> Option<tachys::renderer::mock_dom::Element> {
        None
    }

    /// Returns None (element not found).
    pub fn get_element_by_id(
        &self,
        _id: &str,
    ) -> Option<tachys::renderer::mock_dom::Element> {
        None
    }

    /// Sets the title (no-op in mock mode).
    pub fn set_title(&self, _title: &str) {}
}

/// Stub location function for mock_dom.
pub fn location() -> MockLocation {
    MockLocation
}

/// Mock location type for mock_dom.
#[derive(Clone, Debug)]
pub struct MockLocation;

impl MockLocation {
    /// Returns the hash portion of the URL.
    pub fn hash(&self) -> Result<String, ()> {
        Ok(String::new())
    }

    /// Returns the pathname portion of the URL.
    pub fn pathname(&self) -> Result<String, ()> {
        Ok(String::from("/"))
    }

    /// Returns the origin of the URL.
    pub fn origin(&self) -> Result<String, ()> {
        Ok(String::from("http://localhost"))
    }

    /// Returns the full URL.
    pub fn href(&self) -> Result<String, ()> {
        Ok(String::from("http://localhost/"))
    }

    /// Sets the URL (no-op in mock mode).
    pub fn set_href(&self, _href: &str) -> Result<(), ()> {
        Ok(())
    }
}

/// Returns the current location hash without the beginning #.
pub fn location_hash() -> Option<String> {
    Some(String::new())
}

/// Returns the current pathname.
pub fn location_pathname() -> Option<String> {
    Some(String::from("/"))
}

/// Stub for is_server check.
pub fn is_server() -> bool {
    false
}

/// Stub for is_browser check.
pub fn is_browser() -> bool {
    true
}

/// Handle for animation frame requests.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AnimationFrameRequestHandle(i32);

impl AnimationFrameRequestHandle {
    /// Cancels the animation frame request (no-op in mock mode).
    pub fn cancel(&self) {}
}

/// Stub for request_animation_frame - runs callback immediately in mock mode.
pub fn request_animation_frame(cb: impl FnOnce() + 'static) {
    cb();
}

/// Stub for request_animation_frame_with_handle.
pub fn request_animation_frame_with_handle(
    cb: impl FnOnce() + 'static,
) -> Result<AnimationFrameRequestHandle, ()> {
    cb();
    Ok(AnimationFrameRequestHandle(0))
}

/// Handle for idle callbacks.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdleCallbackHandle(u32);

impl IdleCallbackHandle {
    /// Cancels the idle callback (no-op in mock mode).
    pub fn cancel(&self) {}
}

/// Stub for request_idle_callback - runs callback immediately.
pub fn request_idle_callback(cb: impl Fn() + 'static) {
    cb();
}

/// Stub for request_idle_callback_with_handle.
pub fn request_idle_callback_with_handle(
    cb: impl Fn() + 'static,
) -> Result<IdleCallbackHandle, ()> {
    cb();
    Ok(IdleCallbackHandle(0))
}

/// Queue microtask stub - runs immediately.
pub fn queue_microtask(task: impl FnOnce() + 'static) {
    task();
}

/// Handle for timeouts.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TimeoutHandle(i32);

impl TimeoutHandle {
    /// Clears the timeout (no-op in mock mode).
    pub fn clear(&self) {}
}

/// Stub for set_timeout - runs callback immediately in mock mode.
pub fn set_timeout(cb: impl FnOnce() + 'static, _duration: Duration) {
    cb();
}

/// Stub for set_timeout_with_handle.
pub fn set_timeout_with_handle(
    cb: impl FnOnce() + 'static,
    _duration: Duration,
) -> Result<TimeoutHandle, ()> {
    cb();
    Ok(TimeoutHandle(0))
}

/// Stub debounce function - calls immediately.
pub fn debounce<T: 'static>(
    _delay: Duration,
    mut cb: impl FnMut(T) + 'static,
) -> impl FnMut(T) {
    move |arg| cb(arg)
}

/// Handle for intervals.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntervalHandle(i32);

impl IntervalHandle {
    /// Clears the interval (no-op in mock mode).
    pub fn clear(&self) {}
}

/// Stub for set_interval - does nothing in mock mode (can't simulate repeated calls).
pub fn set_interval(_cb: impl Fn() + 'static, _duration: Duration) {}

/// Stub for set_interval_with_handle.
pub fn set_interval_with_handle(
    _cb: impl Fn() + 'static,
    _duration: Duration,
) -> Result<IntervalHandle, ()> {
    Ok(IntervalHandle(0))
}

/// Handle for window event listeners.
pub struct WindowListenerHandle(Box<dyn FnOnce() + Send + Sync>);

impl core::fmt::Debug for WindowListenerHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("WindowListenerHandle").finish()
    }
}

impl WindowListenerHandle {
    /// Removes the event listener (no-op in mock mode).
    pub fn remove(self) {
        (self.0)()
    }
}

/// Stub for window_event_listener_untyped - does nothing in mock mode.
pub fn window_event_listener_untyped(
    _event_name: &str,
    _cb: impl Fn(MockEvent) + 'static,
) -> WindowListenerHandle {
    WindowListenerHandle(Box::new(|| ()))
}

/// Stub for window_event_listener - does nothing in mock mode.
pub fn window_event_listener<E: EventDescriptor + 'static>(
    _event: E,
    _cb: impl Fn(MockEvent) + 'static,
) -> WindowListenerHandle {
    WindowListenerHandle(Box::new(|| ()))
}

/// Mock event type for mock_dom.
#[derive(Clone, Debug)]
pub struct MockEvent;
