//! Mock-compatible helper functions for use with mock_dom testing.
//!
//! These functions provide the same API as `leptos_dom::helpers` but work
//! with mock event types instead of web_sys types.

use tachys::html::event as ev;

/// Helper function to extract `event.target.value` from an event.
///
/// This is the mock_dom version that extracts the value from the mock event's
/// input data. For input events dispatched via `testing::input(el, value)`,
/// the value will be in the input.data field.
pub fn event_target_value<T: MockEventValue>(event: &T) -> String {
    event.mock_value()
}

/// Helper function to extract `event.target.checked` from an event.
///
/// In mock_dom, this always returns `false` unless the mock event is specifically
/// constructed with checked state (not yet implemented).
pub fn event_target_checked<T>(_event: &T) -> bool {
    // TODO: Add checked state to MockInputData if needed
    false
}

/// Trait for extracting values from mock events.
///
/// This trait is implemented for all mock event types, allowing them to be
/// used with `event_target_value`.
pub trait MockEventValue {
    /// Get the mock value from this event.
    fn mock_value(&self) -> String;
}

// Implement for all mock event types using the input_value() method
macro_rules! impl_mock_event_value {
    ($($ty:ident),*) => {
        $(
            impl MockEventValue for ev::$ty {
                fn mock_value(&self) -> String {
                    self.input_value()
                }
            }
        )*
    };
}

impl_mock_event_value!(
    Event,
    MouseEvent,
    KeyboardEvent,
    FocusEvent,
    InputEvent,
    WheelEvent,
    PointerEvent,
    TouchEvent,
    DragEvent,
    CompositionEvent,
    AnimationEvent,
    TransitionEvent,
    ClipboardEvent,
    HashChangeEvent,
    PageTransitionEvent,
    PopStateEvent,
    ProgressEvent,
    SecurityPolicyViolationEvent,
    StorageEvent,
    SubmitEvent,
    UiEvent,
    BeforeUnloadEvent,
    DeviceMotionEvent,
    DeviceOrientationEvent,
    ErrorEvent,
    GamepadEvent,
    PromiseRejectionEvent,
    CustomEvent,
    MessageEvent
);
