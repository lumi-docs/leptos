#![allow(unused)]

//! A stupidly-simple mock DOM implementation that can be used for testing.
//!
//! Do not use this for anything real. This module provides a simple mock DOM
//! that can be used for unit testing renderer methods without requiring a browser.
//!
//! For component snapshot testing, use SSR (server-side rendering) instead.

use super::CastFrom;
use crate::view::Mountable;
use indexmap::IndexMap;
use slotmap::{new_key_type, SlotMap};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

/// A mock DOM renderer for testing.
///
/// This is intended as a rendering utility for unit testing individual
/// renderer functions without requiring a browser environment.
#[derive(Debug, Copy, Clone)]
pub struct MockDom;

new_key_type! {
    /// A unique identifier for a mock DOM node.
    pub struct NodeId;
}

/// A mock DOM node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Node(pub NodeId);

/// A mock element.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Element(pub Node);

/// A mock text node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Text(pub Node);

/// A mock comment node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Placeholder(pub Node);

/// The current phase of event propagation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventPhase {
    /// No event is being processed.
    #[default]
    None = 0,
    /// Capture phase: event travels from root to target.
    Capturing = 1,
    /// The event has reached the target element.
    AtTarget = 2,
    /// Bubble phase: event travels from target back to root.
    Bubbling = 3,
}

/// Mock event data that can represent any type of DOM event.
#[derive(Clone, Debug, Default)]
pub struct MockEvent {
    /// The event type name (e.g., "click", "keydown")
    pub event_type: String,
    /// The target element ID
    pub target: Option<NodeId>,
    /// Mouse event data
    pub mouse: Option<MockMouseData>,
    /// Keyboard event data
    pub keyboard: Option<MockKeyboardData>,
    /// Input event data
    pub input: Option<MockInputData>,
    /// Focus event data
    pub focus: Option<MockFocusData>,
    /// Wheel event data
    pub wheel: Option<MockWheelData>,
    /// Touch event data
    pub touch: Option<MockTouchData>,
    /// Animation event data
    pub animation: Option<MockAnimationData>,
    /// Transition event data
    pub transition: Option<MockTransitionData>,
    /// Whether default was prevented
    pub default_prevented: std::cell::Cell<bool>,
    /// Whether propagation was stopped
    pub propagation_stopped: std::cell::Cell<bool>,
    /// Whether immediate propagation was stopped (prevents other listeners on same element)
    pub immediate_propagation_stopped: std::cell::Cell<bool>,
    /// The current element handling the event (changes during propagation)
    pub current_target: std::cell::Cell<Option<NodeId>>,
    /// The current phase of event propagation
    pub event_phase: std::cell::Cell<EventPhase>,
    /// Whether this event bubbles
    pub bubbles: bool,
    /// Whether this event is cancelable
    pub cancelable: bool,
    /// Timestamp of the event
    pub time_stamp: f64,
}

/// Mouse event data (for click, mousedown, mousemove, etc.)
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockMouseData {
    pub client_x: i32,
    pub client_y: i32,
    pub page_x: f64,
    pub page_y: f64,
    pub screen_x: i32,
    pub screen_y: i32,
    pub offset_x: f64,
    pub offset_y: f64,
    pub movement_x: i32,
    pub movement_y: i32,
    pub button: i16,
    pub buttons: u16,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
    /// For pointer events
    pub pointer_id: i32,
    pub width: f64,
    pub height: f64,
    pub pressure: f32,
    pub tilt_x: i32,
    pub tilt_y: i32,
    pub pointer_type: String,
    pub is_primary: bool,
}

/// Keyboard event data (for keydown, keyup, keypress)
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockKeyboardData {
    pub key: String,
    pub code: String,
    pub location: u32,
    pub repeat: bool,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
    pub is_composing: bool,
    pub char_code: u32,
    pub key_code: u32,
}

/// Input event data (for input, change)
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockInputData {
    pub data: Option<String>,
    pub input_type: String,
    pub is_composing: bool,
}

/// Focus event data (for focus, blur, focusin, focusout)
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockFocusData {
    pub related_target: Option<NodeId>,
}

/// Wheel event data (for wheel)
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockWheelData {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub delta_mode: u32,
}

/// Touch point data
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockTouch {
    pub identifier: i32,
    pub client_x: i32,
    pub client_y: i32,
    pub page_x: i32,
    pub page_y: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub rotation_angle: f32,
    pub force: f32,
}

/// Touch event data
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockTouchData {
    pub touches: Vec<MockTouch>,
    pub target_touches: Vec<MockTouch>,
    pub changed_touches: Vec<MockTouch>,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
}

/// Animation event data
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockAnimationData {
    pub animation_name: String,
    pub elapsed_time: f32,
    pub pseudo_element: String,
}

/// Transition event data
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct MockTransitionData {
    pub property_name: String,
    pub elapsed_time: f32,
    pub pseudo_element: String,
}

/// The mock event type alias
pub type Event = MockEvent;

/// Mock class list that stores a reference to the element.
#[derive(Clone, Debug)]
pub struct ClassList {
    element_id: NodeId,
}

/// Mock CSS style declaration that stores a reference to the element.
#[derive(Clone, Debug)]
pub struct CssStyleDeclaration {
    element_id: NodeId,
}

/// Mock template element.
#[derive(Clone, Debug, Default)]
pub struct TemplateElement;

impl AsRef<Node> for Node {
    fn as_ref(&self) -> &Node {
        self
    }
}

impl AsRef<Node> for Element {
    fn as_ref(&self) -> &Node {
        &self.0
    }
}

impl AsRef<Node> for Text {
    fn as_ref(&self) -> &Node {
        &self.0
    }
}

impl AsRef<Node> for Placeholder {
    fn as_ref(&self) -> &Node {
        &self.0
    }
}

/// Tests whether two nodes are references to the same underlying node.
pub fn node_eq(a: impl AsRef<Node>, b: impl AsRef<Node>) -> bool {
    a.as_ref() == b.as_ref()
}

impl From<Text> for Node {
    fn from(value: Text) -> Self {
        Node(value.0 .0)
    }
}

impl From<Element> for Node {
    fn from(value: Element) -> Self {
        Node(value.0 .0)
    }
}

impl From<Placeholder> for Node {
    fn from(value: Placeholder) -> Self {
        Node(value.0 .0)
    }
}

impl Element {
    /// Outputs an HTML form of the element, for testing and debugging purposes.
    pub fn to_debug_html(&self) -> String {
        let mut buf = String::new();
        self.debug_html(&mut buf);
        buf
    }

    /// Gets the tag name of this element.
    pub fn tag_name(&self) -> String {
        Document::with_node(self.0 .0, |node| match &node.ty {
            NodeType::Element { tag, .. } => tag.to_string().to_uppercase(),
            _ => String::new(),
        })
        .unwrap_or_default()
    }

    /// Returns HTML with computed styles for snapshot testing.
    ///
    /// Unlike `to_debug_html()`, this includes all CSS cascade-resolved styles,
    /// making it suitable for visual regression testing with insta.
    ///
    /// The output is:
    /// - Indented (2 spaces per level) for readable diffs
    /// - Deterministically ordered (sorted attributes)
    /// - Includes computed styles from loaded stylesheets
    pub fn to_snapshot_html(&self) -> String {
        let mut buf = String::new();
        self.snapshot_html(&mut buf, 0);
        buf
    }

    fn snapshot_html(&self, buf: &mut String, indent: usize) {
        Document::with_node(self.0 .0, |node| {
            snapshot_node(node, self, buf, indent);
        });
    }

    /// Attempt to borrow this element as a specific type (e.g., HtmlElement).
    ///
    /// In mock mode, returns an owned value rather than a reference.
    /// This shadows the JsCast trait method for Elements.
    pub fn dyn_ref<T: events::FromElement>(&self) -> Option<T> {
        T::from_element(self)
    }
}

/// HTML void elements that should not have a closing tag.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

fn snapshot_node(
    node: &NodeData,
    element: &Element,
    buf: &mut String,
    indent: usize,
) {
    match &node.ty {
        NodeType::Text(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                write_indent(buf, indent);
                buf.push_str(trimmed);
                buf.push('\n');
            }
        }
        NodeType::Element {
            tag,
            attrs,
            children,
            classes,
            ..
        } => {
            write_indent(buf, indent);
            buf.push('<');
            buf.push_str(tag);

            // Render classes as class attribute if any
            if !classes.is_empty() {
                buf.push_str(" class=\"");
                for (i, class) in classes.iter().enumerate() {
                    if i > 0 {
                        buf.push(' ');
                    }
                    buf.push_str(class);
                }
                buf.push('"');
            }

            // Render other attributes sorted alphabetically (deterministic)
            let mut sorted_attrs: Vec<_> = attrs
                .iter()
                .filter(|(k, _)| *k != "class" && *k != "style")
                .collect();
            sorted_attrs.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (k, v) in sorted_attrs {
                buf.push(' ');
                buf.push_str(k);
                buf.push_str("=\"");
                buf.push_str(v);
                buf.push('"');
            }

            // Render computed styles (from CSS cascade + inline)
            let computed_styles = testing::get_all_computed_styles(element);
            if !computed_styles.is_empty() {
                buf.push_str(" style=\"");
                for (i, (k, v)) in computed_styles.iter().enumerate() {
                    if i > 0 {
                        buf.push_str("; ");
                    }
                    buf.push_str(k);
                    buf.push_str(": ");
                    buf.push_str(v);
                }
                buf.push('"');
            }

            // Handle void elements
            let is_void = VOID_ELEMENTS.contains(&tag.to_lowercase().as_str());
            if is_void {
                buf.push_str(" />\n");
            } else {
                buf.push('>');

                // Render children
                if children.is_empty() {
                    buf.push_str("</");
                    buf.push_str(tag);
                    buf.push_str(">\n");
                } else {
                    buf.push('\n');
                    for child in children {
                        snapshot_child(child, buf, indent + 2);
                    }
                    write_indent(buf, indent);
                    buf.push_str("</");
                    buf.push_str(tag);
                    buf.push_str(">\n");
                }
            }
        }
        NodeType::Placeholder => {
            write_indent(buf, indent);
            buf.push_str("<!>\n");
        }
    }
}

fn snapshot_child(node: &Node, buf: &mut String, indent: usize) {
    Document::with_node(node.0, |node_data| {
        match &node_data.ty {
            NodeType::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    write_indent(buf, indent);
                    buf.push_str(trimmed);
                    buf.push('\n');
                }
            }
            NodeType::Element {
                tag,
                attrs,
                children,
                classes,
                ..
            } => {
                write_indent(buf, indent);
                buf.push('<');
                buf.push_str(tag);

                // Render classes
                if !classes.is_empty() {
                    buf.push_str(" class=\"");
                    for (i, class) in classes.iter().enumerate() {
                        if i > 0 {
                            buf.push(' ');
                        }
                        buf.push_str(class);
                    }
                    buf.push('"');
                }

                // Render sorted attributes
                let mut sorted_attrs: Vec<_> = attrs
                    .iter()
                    .filter(|(k, _)| *k != "class" && *k != "style")
                    .collect();
                sorted_attrs.sort_by(|(a, _), (b, _)| a.cmp(b));
                for (k, v) in sorted_attrs {
                    buf.push(' ');
                    buf.push_str(k);
                    buf.push_str("=\"");
                    buf.push_str(v);
                    buf.push('"');
                }

                // For child elements, we need to get computed styles
                // We need to find the Element wrapper for this node
                let child_element = Element(Node(node.0));
                let computed_styles =
                    testing::get_all_computed_styles(&child_element);
                if !computed_styles.is_empty() {
                    buf.push_str(" style=\"");
                    for (i, (k, v)) in computed_styles.iter().enumerate() {
                        if i > 0 {
                            buf.push_str("; ");
                        }
                        buf.push_str(k);
                        buf.push_str(": ");
                        buf.push_str(v);
                    }
                    buf.push('"');
                }

                let is_void =
                    VOID_ELEMENTS.contains(&tag.to_lowercase().as_str());
                if is_void {
                    buf.push_str(" />\n");
                } else {
                    buf.push('>');

                    if children.is_empty() {
                        buf.push_str("</");
                        buf.push_str(tag);
                        buf.push_str(">\n");
                    } else {
                        buf.push('\n');
                        for child in children {
                            snapshot_child(child, buf, indent + 2);
                        }
                        write_indent(buf, indent);
                        buf.push_str("</");
                        buf.push_str(tag);
                        buf.push_str(">\n");
                    }
                }
            }
            NodeType::Placeholder => {
                write_indent(buf, indent);
                buf.push_str("<!>\n");
            }
        }
    });
}

fn write_indent(buf: &mut String, indent: usize) {
    for _ in 0..indent {
        buf.push(' ');
    }
}

/// The DOM data associated with a particular node.
#[derive(Debug, PartialEq, Eq)]
pub struct NodeData {
    /// The node's parent.
    pub parent: Option<NodeId>,
    /// The node itself.
    pub ty: NodeType,
}

trait DebugHtml {
    fn debug_html(&self, buf: &mut String);
}

impl DebugHtml for Element {
    fn debug_html(&self, buf: &mut String) {
        Document::with_node(self.0 .0, |node| {
            node.debug_html(buf);
        });
    }
}

impl DebugHtml for Text {
    fn debug_html(&self, buf: &mut String) {
        Document::with_node(self.0 .0, |node| {
            node.debug_html(buf);
        });
    }
}

impl DebugHtml for Node {
    fn debug_html(&self, buf: &mut String) {
        Document::with_node(self.0, |node| {
            node.debug_html(buf);
        });
    }
}

impl DebugHtml for NodeData {
    fn debug_html(&self, buf: &mut String) {
        match &self.ty {
            NodeType::Text(text) => buf.push_str(text),
            NodeType::Element {
                tag,
                attrs,
                children,
                classes,
                inline_styles,
            } => {
                buf.push('<');
                buf.push_str(tag);

                // Render classes as class attribute if any
                if !classes.is_empty() {
                    buf.push_str(" class=\"");
                    for (i, class) in classes.iter().enumerate() {
                        if i > 0 {
                            buf.push(' ');
                        }
                        buf.push_str(class);
                    }
                    buf.push('"');
                }

                // Render inline styles as style attribute if any
                if !inline_styles.is_empty() {
                    buf.push_str(" style=\"");
                    for (i, (k, v)) in inline_styles.iter().enumerate() {
                        if i > 0 {
                            buf.push_str("; ");
                        }
                        buf.push_str(k);
                        buf.push_str(": ");
                        buf.push_str(v);
                    }
                    buf.push('"');
                }

                // Render other attributes (skip class and style since we handle them separately)
                for (k, v) in attrs {
                    if k != "class" && k != "style" {
                        buf.push(' ');
                        buf.push_str(k);
                        buf.push_str("=\"");
                        buf.push_str(v);
                        buf.push('"');
                    }
                }
                buf.push('>');

                for child in children {
                    child.debug_html(buf);
                }

                buf.push_str("</");
                buf.push_str(tag);
                buf.push('>');
            }
            NodeType::Placeholder => buf.push_str("<!>"),
        }
    }
}

/// The mock DOM document.
#[derive(Clone)]
pub struct Document(Rc<RefCell<SlotMap<NodeId, NodeData>>>);

impl Document {
    /// Creates a new document.
    pub fn new() -> Self {
        Document(Default::default())
    }

    fn with_node<U>(id: NodeId, f: impl FnOnce(&NodeData) -> U) -> Option<U> {
        DOCUMENT.with(|d| {
            let data = d.0.borrow();
            let data = data.get(id);
            data.map(f)
        })
    }

    fn with_node_mut<U>(
        id: NodeId,
        f: impl FnOnce(&mut NodeData) -> U,
    ) -> Option<U> {
        DOCUMENT.with(|d| {
            let mut data = d.0.borrow_mut();
            let data = data.get_mut(id);
            data.map(f)
        })
    }

    /// Resets the document's contents.
    pub fn reset(&self) {
        self.0.borrow_mut().clear();
    }

    /// Creates a new element with the given tag name.
    pub fn create_element(&self, tag: &str) -> Element {
        Element(Node(self.0.borrow_mut().insert(NodeData {
            parent: None,
            ty: NodeType::Element {
                tag: tag.to_string().into(),
                attrs: IndexMap::new(),
                children: Vec::new(),
                classes: Vec::new(),
                inline_styles: IndexMap::new(),
            },
        })))
    }

    fn create_text_node(&self, data: &str) -> Text {
        Text(Node(self.0.borrow_mut().insert(NodeData {
            parent: None,
            ty: NodeType::Text(data.to_string()),
        })))
    }

    fn create_placeholder(&self) -> Placeholder {
        Placeholder(Node(self.0.borrow_mut().insert(NodeData {
            parent: None,
            ty: NodeType::Placeholder,
        })))
    }

    /// Query for an element matching the given CSS selector.
    ///
    /// Returns the first element that matches, or None if no match found.
    /// Uses the selector parsing infrastructure from the testing module.
    pub fn query_selector(selector: &str) -> Option<Element> {
        DOCUMENT.with(|d| {
            let data = d.0.borrow();
            // Iterate through all nodes and find matching elements
            for (id, node) in data.iter() {
                if matches!(&node.ty, NodeType::Element { .. }) {
                    let element = Element(Node(id));
                    if testing::matches_selector(&element, selector) {
                        return Some(element);
                    }
                }
            }
            None
        })
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static DOCUMENT: Document = Document::new();
}

/// Returns the global document.
pub fn document() -> Document {
    DOCUMENT.with(Clone::clone)
}

/// Resets the global document for a fresh test.
/// Also clears event listeners, recorded events, and loaded stylesheets.
pub fn reset_document() {
    DOCUMENT.with(|d| d.reset());
    clear_event_listeners();
    testing::clear_recorded_events();
    testing::disable_recording();
    testing::clear_stylesheets();
}

// ============================================================================
// Event Listener Storage
// ============================================================================

type EventCallback = Rc<RefCell<dyn FnMut(MockEvent)>>;

struct StoredListener {
    event_name: String,
    callback: EventCallback,
    #[allow(dead_code)]
    use_capture: bool,
}

thread_local! {
    static EVENT_LISTENERS: RefCell<HashMap<NodeId, Vec<StoredListener>>> =
        RefCell::new(HashMap::new());
}

/// Clears all stored event listeners (useful between tests).
pub fn clear_event_listeners() {
    EVENT_LISTENERS.with(|listeners| {
        listeners.borrow_mut().clear();
    });
}

// ============================================================================
// Testing Utilities
// ============================================================================

/// Testing utilities for dispatching events in mock_dom tests.
pub mod testing {
    use super::*;

    // ========================================================================
    // Snapshot Testing
    // ========================================================================

    /// Returns snapshot HTML for use with insta.
    ///
    /// This produces a complete, deterministic representation of the element
    /// including computed styles from loaded CSS. Any change that affects
    /// visual appearance will cause the snapshot to change.
    ///
    /// # Example
    ///
    /// ```ignore
    /// reset_document();
    /// testing::load_stylesheet(include_str!("../compiled.css"));
    ///
    /// let button = create_my_component();
    /// insta::assert_snapshot!(testing::snapshot(&button));
    /// ```
    pub fn snapshot(element: &Element) -> String {
        element.to_snapshot_html()
    }

    // ========================================================================
    // Event Recording
    // ========================================================================

    /// A recorded event for test verification.
    #[derive(Clone, Debug)]
    pub struct RecordedEvent {
        /// The event type (e.g., "click", "keydown")
        pub event_type: String,
        /// The target element ID
        pub target: NodeId,
        /// The full event data
        pub event: MockEvent,
        /// Timestamp when the event was dispatched
        pub timestamp: std::time::Instant,
    }

    thread_local! {
        /// Recorded events for test verification.
        static RECORDED_EVENTS: RefCell<Vec<RecordedEvent>> = const { RefCell::new(Vec::new()) };
        /// Whether event recording is enabled (default: false for performance).
        static RECORDING_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    /// Enable event recording for tests.
    pub fn enable_recording() {
        RECORDING_ENABLED.with(|enabled| enabled.set(true));
    }

    /// Disable event recording.
    pub fn disable_recording() {
        RECORDING_ENABLED.with(|enabled| enabled.set(false));
    }

    /// Check if recording is enabled.
    pub fn is_recording() -> bool {
        RECORDING_ENABLED.with(|enabled| enabled.get())
    }

    /// Get the count of recorded events for a specific element and event type.
    pub fn event_count(el: &Element, event_type: &str) -> usize {
        let node_id = el.0 .0;
        RECORDED_EVENTS.with(|events| {
            events
                .borrow()
                .iter()
                .filter(|e| e.target == node_id && e.event_type == event_type)
                .count()
        })
    }

    /// Take all recorded events for a specific element, clearing them.
    pub fn take_events(el: &Element) -> Vec<RecordedEvent> {
        let node_id = el.0 .0;
        RECORDED_EVENTS.with(|events| {
            let mut events = events.borrow_mut();
            let (matching, remaining): (Vec<_>, Vec<_>) =
                events.drain(..).partition(|e| e.target == node_id);
            *events = remaining;
            matching
        })
    }

    /// Take all recorded events of a specific type for an element.
    pub fn take_events_of_type(
        el: &Element,
        event_type: &str,
    ) -> Vec<RecordedEvent> {
        let node_id = el.0 .0;
        RECORDED_EVENTS.with(|events| {
            let mut events = events.borrow_mut();
            let (matching, remaining): (Vec<_>, Vec<_>) =
                events.drain(..).partition(|e| {
                    e.target == node_id && e.event_type == event_type
                });
            *events = remaining;
            matching
        })
    }

    /// Get a clone of all recorded events.
    pub fn all_events() -> Vec<RecordedEvent> {
        RECORDED_EVENTS.with(|events| events.borrow().clone())
    }

    /// Clear all recorded events.
    pub fn clear_recorded_events() {
        RECORDED_EVENTS.with(|events| events.borrow_mut().clear());
    }

    /// Get total count of all recorded events.
    pub fn total_event_count() -> usize {
        RECORDED_EVENTS.with(|events| events.borrow().len())
    }

    // ========================================================================
    // Event Expectations (like wiremock)
    // ========================================================================

    /// An expectation for event verification.
    pub struct EventExpectation {
        element: NodeId,
        event_type: String,
        expected_count: Option<usize>,
        min_count: Option<usize>,
        max_count: Option<usize>,
    }

    impl EventExpectation {
        /// Create an expectation for an element and event type.
        pub fn new(el: &Element, event_type: &str) -> Self {
            Self {
                element: el.0 .0,
                event_type: event_type.to_string(),
                expected_count: None,
                min_count: None,
                max_count: None,
            }
        }

        /// Expect exactly n events.
        pub fn times(mut self, n: usize) -> Self {
            self.expected_count = Some(n);
            self
        }

        /// Expect at least n events.
        pub fn at_least(mut self, n: usize) -> Self {
            self.min_count = Some(n);
            self
        }

        /// Expect at most n events.
        pub fn at_most(mut self, n: usize) -> Self {
            self.max_count = Some(n);
            self
        }

        /// Verify the expectation against recorded events.
        pub fn verify(&self) {
            let count = RECORDED_EVENTS.with(|events| {
                events
                    .borrow()
                    .iter()
                    .filter(|e| {
                        e.target == self.element
                            && e.event_type == self.event_type
                    })
                    .count()
            });

            if let Some(expected) = self.expected_count {
                assert_eq!(
                    count, expected,
                    "Expected {} '{}' events, got {}",
                    expected, self.event_type, count
                );
            }
            if let Some(min) = self.min_count {
                assert!(
                    count >= min,
                    "Expected at least {} '{}' events, got {}",
                    min,
                    self.event_type,
                    count
                );
            }
            if let Some(max) = self.max_count {
                assert!(
                    count <= max,
                    "Expected at most {} '{}' events, got {}",
                    max,
                    self.event_type,
                    count
                );
            }
        }
    }

    /// Helper to create an expectation.
    pub fn expect_event(el: &Element, event_type: &str) -> EventExpectation {
        EventExpectation::new(el, event_type)
    }

    // ========================================================================
    // Event Propagation
    // ========================================================================

    /// Build path from target to root: [target, parent, ..., root]
    fn build_propagation_path(target_id: NodeId) -> Vec<NodeId> {
        let mut path = vec![target_id];
        let mut current = target_id;

        while let Some(parent_id) =
            Document::with_node(current, |node| node.parent).flatten()
        {
            path.push(parent_id);
            current = parent_id;
        }

        path
    }

    /// Fire listeners for capture or bubble phase.
    /// Returns false if propagation was stopped.
    fn fire_listeners_for_phase(
        node_id: NodeId,
        event: &MockEvent,
        capture: bool,
    ) -> bool {
        EVENT_LISTENERS.with(|listeners| {
            let listeners = listeners.borrow();
            if let Some(list) = listeners.get(&node_id) {
                for listener in list {
                    if listener.event_name == event.event_type
                        && listener.use_capture == capture
                    {
                        if event.immediate_propagation_stopped.get() {
                            return false;
                        }
                        let mut cb = listener.callback.borrow_mut();
                        cb(event.clone());
                        if event.propagation_stopped.get() {
                            return false;
                        }
                    }
                }
            }
            true
        })
    }

    /// Fire all listeners at target (both capture and non-capture).
    fn fire_listeners_at_target(node_id: NodeId, event: &MockEvent) {
        EVENT_LISTENERS.with(|listeners| {
            let listeners = listeners.borrow();
            if let Some(list) = listeners.get(&node_id) {
                for listener in list {
                    if listener.event_name == event.event_type {
                        if event.immediate_propagation_stopped.get() {
                            return;
                        }
                        let mut cb = listener.callback.borrow_mut();
                        cb(event.clone());
                    }
                }
            }
        });
    }

    /// Dispatch an event with full DOM propagation (capture, target, bubble phases).
    ///
    /// This implements the W3C DOM event flow:
    /// 1. **Capture phase**: Event travels from root to target (listeners with `use_capture=true`)
    /// 2. **Target phase**: All listeners on the target element fire
    /// 3. **Bubble phase**: Event travels from target back to root (if `event.bubbles`)
    ///
    /// After propagation, automatically polls the executor to process reactive updates.
    pub fn dispatch_event(el: &Element, event: MockEvent) {
        let target_id = el.0 .0;

        // Record the event if recording is enabled
        if is_recording() {
            RECORDED_EVENTS.with(|events| {
                events.borrow_mut().push(RecordedEvent {
                    event_type: event.event_type.clone(),
                    target: target_id,
                    event: event.clone(),
                    timestamp: std::time::Instant::now(),
                });
            });
        }

        // Set up event state
        event.current_target.set(Some(target_id));

        // Build the propagation path: [target, parent, ..., root]
        let path = build_propagation_path(target_id);

        // Phase 1: Capture (root -> target, excluding target)
        event.event_phase.set(EventPhase::Capturing);
        for &node_id in path.iter().rev().skip(1) {
            // Skip target, go root->parent
            event.current_target.set(Some(node_id));
            if !fire_listeners_for_phase(node_id, &event, true) {
                break;
            }
        }

        // Phase 2: Target (fire all listeners on target)
        if !event.propagation_stopped.get() {
            event.event_phase.set(EventPhase::AtTarget);
            event.current_target.set(Some(target_id));
            fire_listeners_at_target(target_id, &event);
        }

        // Phase 3: Bubble (target -> root, excluding target)
        if event.bubbles && !event.propagation_stopped.get() {
            event.event_phase.set(EventPhase::Bubbling);
            for &node_id in path.iter().skip(1) {
                // Skip target, go parent->root
                event.current_target.set(Some(node_id));
                if !fire_listeners_for_phase(node_id, &event, false) {
                    break;
                }
            }
        }

        event.event_phase.set(EventPhase::None);

        // Process any reactive updates spawned by the event handlers.
        any_spawner::Executor::poll_local();
    }

    /// Process pending reactive updates without dispatching an event.
    ///
    /// Call this after programmatically updating signals (without going through
    /// an event handler) to flush reactive updates to the DOM.
    ///
    /// # Example
    /// ```ignore
    /// let count = RwSignal::new(0);
    /// // ... build view with count ...
    /// count.set(1);
    /// testing::tick();  // Process the reactive update
    /// assert_eq!(el.to_debug_html(), "...");
    /// ```
    pub fn tick() {
        any_spawner::Executor::poll_local();
    }

    /// Simulate a click event on an element.
    pub fn click(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "click".to_string(),
                target: Some(el.0 .0),
                mouse: Some(MockMouseData::default()),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate a click event at specific coordinates.
    pub fn click_at(el: &Element, x: i32, y: i32) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "click".to_string(),
                target: Some(el.0 .0),
                mouse: Some(MockMouseData {
                    client_x: x,
                    client_y: y,
                    page_x: x as f64,
                    page_y: y as f64,
                    ..Default::default()
                }),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate a mousedown event.
    pub fn mousedown(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "mousedown".to_string(),
                target: Some(el.0 .0),
                mouse: Some(MockMouseData::default()),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate a mouseup event.
    pub fn mouseup(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "mouseup".to_string(),
                target: Some(el.0 .0),
                mouse: Some(MockMouseData::default()),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate a keydown event.
    pub fn keydown(el: &Element, key: &str) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "keydown".to_string(),
                target: Some(el.0 .0),
                keyboard: Some(MockKeyboardData {
                    key: key.to_string(),
                    ..Default::default()
                }),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate a keyup event.
    pub fn keyup(el: &Element, key: &str) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "keyup".to_string(),
                target: Some(el.0 .0),
                keyboard: Some(MockKeyboardData {
                    key: key.to_string(),
                    ..Default::default()
                }),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate an input event with a value.
    pub fn input(el: &Element, value: &str) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "input".to_string(),
                target: Some(el.0 .0),
                input: Some(MockInputData {
                    data: Some(value.to_string()),
                    ..Default::default()
                }),
                bubbles: true,
                cancelable: false,
                ..Default::default()
            },
        );
    }

    /// Simulate a change event.
    pub fn change(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "change".to_string(),
                target: Some(el.0 .0),
                bubbles: true,
                cancelable: false,
                ..Default::default()
            },
        );
    }

    /// Simulate a focus event.
    pub fn focus(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "focus".to_string(),
                target: Some(el.0 .0),
                focus: Some(MockFocusData::default()),
                bubbles: false,
                cancelable: false,
                ..Default::default()
            },
        );
    }

    /// Simulate a blur event.
    pub fn blur(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "blur".to_string(),
                target: Some(el.0 .0),
                focus: Some(MockFocusData::default()),
                bubbles: false,
                cancelable: false,
                ..Default::default()
            },
        );
    }

    /// Simulate a submit event.
    pub fn submit(el: &Element) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "submit".to_string(),
                target: Some(el.0 .0),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Simulate a wheel event.
    pub fn wheel(el: &Element, delta_y: f64) {
        dispatch_event(
            el,
            MockEvent {
                event_type: "wheel".to_string(),
                target: Some(el.0 .0),
                wheel: Some(MockWheelData {
                    delta_y,
                    ..Default::default()
                }),
                mouse: Some(MockMouseData::default()),
                bubbles: true,
                cancelable: true,
                ..Default::default()
            },
        );
    }

    /// Builder for creating custom mock events.
    pub struct EventBuilder {
        event: MockEvent,
    }

    impl EventBuilder {
        /// Create a new event builder for the given event type.
        pub fn new(event_type: &str) -> Self {
            Self {
                event: MockEvent {
                    event_type: event_type.to_string(),
                    bubbles: true,
                    cancelable: true,
                    ..Default::default()
                },
            }
        }

        /// Add mouse event data.
        pub fn mouse(mut self) -> Self {
            self.event.mouse = Some(MockMouseData::default());
            self
        }

        /// Add keyboard event data.
        pub fn keyboard(mut self) -> Self {
            self.event.keyboard = Some(MockKeyboardData::default());
            self
        }

        /// Set mouse coordinates.
        pub fn at(mut self, x: i32, y: i32) -> Self {
            if let Some(ref mut m) = self.event.mouse {
                m.client_x = x;
                m.client_y = y;
                m.page_x = x as f64;
                m.page_y = y as f64;
            }
            self
        }

        /// Set mouse button.
        pub fn button(mut self, button: i16) -> Self {
            if let Some(ref mut m) = self.event.mouse {
                m.button = button;
            }
            self
        }

        /// Set keyboard key.
        pub fn key(mut self, key: &str) -> Self {
            if let Some(ref mut k) = self.event.keyboard {
                k.key = key.to_string();
            }
            self
        }

        /// Set keyboard code.
        pub fn code(mut self, code: &str) -> Self {
            if let Some(ref mut k) = self.event.keyboard {
                k.code = code.to_string();
            }
            self
        }

        /// Add ctrl modifier.
        pub fn with_ctrl(mut self) -> Self {
            if let Some(ref mut m) = self.event.mouse {
                m.ctrl_key = true;
            }
            if let Some(ref mut k) = self.event.keyboard {
                k.ctrl_key = true;
            }
            self
        }

        /// Add shift modifier.
        pub fn with_shift(mut self) -> Self {
            if let Some(ref mut m) = self.event.mouse {
                m.shift_key = true;
            }
            if let Some(ref mut k) = self.event.keyboard {
                k.shift_key = true;
            }
            self
        }

        /// Add alt modifier.
        pub fn with_alt(mut self) -> Self {
            if let Some(ref mut m) = self.event.mouse {
                m.alt_key = true;
            }
            if let Some(ref mut k) = self.event.keyboard {
                k.alt_key = true;
            }
            self
        }

        /// Add meta modifier.
        pub fn with_meta(mut self) -> Self {
            if let Some(ref mut m) = self.event.mouse {
                m.meta_key = true;
            }
            if let Some(ref mut k) = self.event.keyboard {
                k.meta_key = true;
            }
            self
        }

        /// Set whether the event bubbles.
        pub fn bubbles(mut self, bubbles: bool) -> Self {
            self.event.bubbles = bubbles;
            self
        }

        /// Set whether the event is cancelable.
        pub fn cancelable(mut self, cancelable: bool) -> Self {
            self.event.cancelable = cancelable;
            self
        }

        /// Dispatch the event to an element.
        pub fn dispatch(mut self, el: &Element) {
            self.event.target = Some(el.0 .0);
            dispatch_event(el, self.event);
        }

        /// Build the event without dispatching.
        pub fn build(self) -> MockEvent {
            self.event
        }
    }

    // ========================================================================
    // CSS Cascade / Computed Styles
    // ========================================================================

    use std::collections::BTreeMap;

    /// A parsed CSS property value with importance tracking.
    #[derive(Clone, Debug)]
    pub struct CssPropertyValue {
        /// The CSS value as a string.
        pub value: String,
        /// Whether this declaration has !important.
        pub important: bool,
    }

    /// A parsed CSS rule with selector and properties.
    #[derive(Clone, Debug)]
    pub struct ParsedRule {
        /// The original selector string (for debugging).
        pub selector_string: String,
        /// Specificity as (a, b, c) tuple.
        pub specificity: (u32, u32, u32),
        /// Properties from this rule (sorted by name for determinism).
        pub properties: BTreeMap<String, CssPropertyValue>,
        /// Source order for cascade tie-breaking.
        pub source_order: usize,
    }

    /// A parsed stylesheet containing rules.
    #[derive(Clone, Debug)]
    pub struct ParsedStylesheet {
        /// Rules sorted by specificity and source order.
        pub rules: Vec<ParsedRule>,
        /// Hash of the original CSS content for change detection.
        pub content_hash: u64,
    }

    thread_local! {
        /// Loaded stylesheets for cascade computation.
        static STYLESHEETS: RefCell<Vec<ParsedStylesheet>> = const { RefCell::new(Vec::new()) };
    }

    /// Load a compiled CSS stylesheet for cascade computation.
    ///
    /// This parses the CSS and stores it for use in `get_computed_style()`.
    /// Call this with your compiled Tailwind CSS output.
    ///
    /// # Example
    /// ```ignore
    /// let css = include_str!("../dist/styles.css");
    /// testing::load_stylesheet(css);
    /// ```
    #[cfg(feature = "css_cascade")]
    pub fn load_stylesheet(css_content: &str) {
        use lightningcss::printer::PrinterOptions;
        use lightningcss::rules::CssRule;
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        use lightningcss::traits::ToCss;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Compute content hash for change detection
        let mut hasher = DefaultHasher::new();
        css_content.hash(&mut hasher);
        let content_hash = hasher.finish();

        // Parse the stylesheet
        let stylesheet =
            match StyleSheet::parse(css_content, ParserOptions::default()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("CSS parse error: {:?}", e);
                    return;
                }
            };

        let mut rules = Vec::new();
        let mut source_order = 0usize;

        // Recursively extract style rules from all nesting levels (@layer, @media, etc.)
        fn extract_rules<'i>(
            css_rules: &lightningcss::rules::CssRuleList<'i>,
            rules: &mut Vec<ParsedRule>,
            source_order: &mut usize,
        ) {
            use lightningcss::printer::PrinterOptions;
            use lightningcss::traits::ToCss;

            for rule in css_rules.0.iter() {
                match rule {
                    CssRule::Style(style_rule) => {
                        // Get selector string
                        let selector_string = style_rule
                            .selectors
                            .to_css_string(PrinterOptions::default())
                            .unwrap_or_default();

                        // Calculate specificity (use first selector's specificity)
                        // Specificity is packed into u32: bits 20-29 = IDs, 10-19 = classes, 0-9 = elements
                        let specificity = if !style_rule.selectors.0.is_empty()
                        {
                            let spec = style_rule.selectors.0[0].specificity();
                            let id_selectors = (spec >> 20) & 0x3FF;
                            let class_selectors = (spec >> 10) & 0x3FF;
                            let element_selectors = spec & 0x3FF;
                            (id_selectors, class_selectors, element_selectors)
                        } else {
                            (0, 0, 0)
                        };

                        // Extract properties
                        let mut properties = BTreeMap::new();

                        // Helper to fix lightningcss infinity serialization bug
                        // Tailwind v4 uses calc(infinity * 1px) for rounded-full
                        // lightningcss converts infinity to f32::MAX (~3.40282e38)
                        fn fix_infinity_overflow(value: &str) -> String {
                            // f32::MAX serializes to various forms depending on locale/version
                            if value.contains("3.40282")
                                && value.contains("e38")
                                || value.contains("e+38")
                            {
                                // Replace any f32::MAX px value with 9999px (large but valid)
                                let mut result = value.to_string();
                                // Handle different scientific notation formats
                                for pattern in &[
                                    "3.40282e38px",
                                    "3.40282e+38px",
                                    "3.40282347e38px",
                                    "3.40282347e+38px",
                                ] {
                                    result = result.replace(pattern, "9999px");
                                }
                                result
                            } else {
                                value.to_string()
                            }
                        }

                        // Process normal declarations
                        for property in
                            style_rule.declarations.declarations.iter()
                        {
                            let name =
                                property.property_id().name().to_string();
                            if let Ok(value) = property
                                .value_to_css_string(PrinterOptions::default())
                            {
                                properties.insert(
                                    name,
                                    CssPropertyValue {
                                        value: fix_infinity_overflow(&value),
                                        important: false,
                                    },
                                );
                            }
                        }

                        // Process !important declarations
                        for property in style_rule
                            .declarations
                            .important_declarations
                            .iter()
                        {
                            let name =
                                property.property_id().name().to_string();
                            if let Ok(value) = property
                                .value_to_css_string(PrinterOptions::default())
                            {
                                properties.insert(
                                    name,
                                    CssPropertyValue {
                                        value: fix_infinity_overflow(&value),
                                        important: true,
                                    },
                                );
                            }
                        }

                        // Only add rules that have properties
                        if !properties.is_empty() {
                            rules.push(ParsedRule {
                                selector_string,
                                specificity,
                                properties,
                                source_order: *source_order,
                            });
                            *source_order += 1;
                        }
                    }
                    CssRule::LayerBlock(layer) => {
                        // Recurse into @layer blocks (Tailwind v4 uses these extensively)
                        extract_rules(&layer.rules, rules, source_order);
                    }
                    CssRule::Media(media) => {
                        // Recurse into @media blocks
                        // Note: We don't check media query conditions, we extract all rules
                        extract_rules(&media.rules, rules, source_order);
                    }
                    CssRule::Supports(supports) => {
                        // Recurse into @supports blocks
                        extract_rules(&supports.rules, rules, source_order);
                    }
                    _ => {
                        // Skip other rule types (@keyframes, @font-face, etc.)
                    }
                }
            }
        }

        extract_rules(&stylesheet.rules, &mut rules, &mut source_order);

        STYLESHEETS.with(|sheets| {
            sheets.borrow_mut().push(ParsedStylesheet {
                rules,
                content_hash,
            });
        });
    }

    /// Stub for when css_cascade feature is disabled.
    #[cfg(not(feature = "css_cascade"))]
    pub fn load_stylesheet(_css_content: &str) {
        // No-op without css_cascade feature
    }

    /// Load a stylesheet from a file path.
    pub fn load_stylesheet_from_file(
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        load_stylesheet(&content);
        Ok(())
    }

    /// Clear all loaded stylesheets.
    pub fn clear_stylesheets() {
        STYLESHEETS.with(|sheets| sheets.borrow_mut().clear());
    }

    /// Get a deterministic hash of all loaded stylesheets.
    ///
    /// Useful for detecting when Tailwind config changes affect the output.
    /// Snapshot this value with insta to detect CSS changes.
    pub fn stylesheet_hash() -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        STYLESHEETS.with(|sheets| {
            let mut hasher = DefaultHasher::new();
            for sheet in sheets.borrow().iter() {
                sheet.content_hash.hash(&mut hasher);
            }
            hasher.finish()
        })
    }

    /// Get the number of loaded stylesheets.
    pub fn stylesheet_count() -> usize {
        STYLESHEETS.with(|sheets| sheets.borrow().len())
    }

    /// Get total number of parsed rules across all stylesheets.
    pub fn rule_count() -> usize {
        STYLESHEETS
            .with(|sheets| sheets.borrow().iter().map(|s| s.rules.len()).sum())
    }

    // ========================================================================
    // Selector Parsing and Matching
    // ========================================================================

    /// Attribute selector operator.
    #[derive(Clone, Debug, Copy, PartialEq, Eq)]
    pub enum AttributeOperator {
        /// `[attr]` - has attribute
        Exists,
        /// `[attr=value]` - exact match
        Equals,
        /// `[attr*=value]` - contains
        Contains,
        /// `[attr^=value]` - starts with
        StartsWith,
        /// `[attr$=value]` - ends with
        EndsWith,
        /// `[attr~=value]` - contains word
        ContainsWord,
        /// `[attr|=value]` - starts with or equals
        DashMatch,
    }

    /// An attribute selector.
    #[derive(Clone, Debug)]
    pub struct AttributeSelector {
        /// Attribute name.
        pub name: String,
        /// Operator (None = just [attr] existence check).
        pub operator: AttributeOperator,
        /// Value to match against (empty for Exists).
        pub value: String,
    }

    /// Combinator between selector parts.
    #[derive(Clone, Debug, Copy, PartialEq, Eq)]
    pub enum Combinator {
        /// Space - descendant (any level)
        Descendant,
        /// `>` - direct child
        Child,
        /// `+` - adjacent sibling (immediately following)
        AdjacentSibling,
        /// `~` - general sibling (any following)
        GeneralSibling,
    }

    /// A single selector part (e.g., `div.class-name[attr]`).
    #[derive(Clone, Debug, Default)]
    pub struct SelectorPart {
        /// Element type (e.g., "div"), None for universal or class-only.
        pub element: Option<String>,
        /// ID selector (without #).
        pub id: Option<String>,
        /// Class selectors (without .).
        pub classes: Vec<String>,
        /// Attribute selectors.
        pub attributes: Vec<AttributeSelector>,
    }

    /// A selector sequence with combinators (e.g., `.parent > .child`).
    #[derive(Clone, Debug)]
    pub struct SelectorSequence {
        /// Parts and their combinators. Last part has no combinator.
        /// Stored right-to-left: [key_selector, parent, grandparent, ...]
        pub parts: Vec<(SelectorPart, Option<Combinator>)>,
    }

    /// A parsed CSS selector (may contain comma-separated groups).
    #[derive(Clone, Debug)]
    pub struct ParsedSelector {
        /// The original selector string.
        pub raw: String,
        /// Selector groups (comma = OR).
        pub groups: Vec<SelectorSequence>,
    }

    /// Parse a CSS selector string into a structured form.
    ///
    /// Supports: class, ID, element, attribute selectors, and combinators.
    pub fn parse_selector(selector: &str) -> ParsedSelector {
        let raw = selector.to_string();
        let groups = selector
            .split(',')
            .map(|s| parse_selector_sequence(s.trim()))
            .collect();

        ParsedSelector { raw, groups }
    }

    /// Parse a single selector sequence (no commas).
    fn parse_selector_sequence(selector: &str) -> SelectorSequence {
        let mut parts = Vec::new();
        let mut current_part = SelectorPart::default();
        let mut chars = selector.chars().peekable();
        let mut pending_combinator: Option<Combinator> = None;

        while let Some(c) = chars.next() {
            match c {
                // Whitespace - potential descendant combinator
                ' ' | '\t' | '\n' | '\r' => {
                    // Skip additional whitespace
                    while chars
                        .peek()
                        .map(|c| c.is_whitespace())
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                    // Check if followed by explicit combinator
                    match chars.peek() {
                        Some('>') | Some('+') | Some('~') => {
                            // Will be handled by explicit combinator
                        }
                        Some(_) => {
                            // Descendant combinator (space)
                            if !current_part.is_empty() {
                                parts.push((
                                    current_part,
                                    Some(Combinator::Descendant),
                                ));
                                current_part = SelectorPart::default();
                            }
                            pending_combinator = None;
                        }
                        None => {}
                    }
                }
                // Child combinator
                '>' => {
                    // Skip whitespace after combinator
                    while chars
                        .peek()
                        .map(|c| c.is_whitespace())
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                    if !current_part.is_empty() {
                        parts.push((current_part, Some(Combinator::Child)));
                        current_part = SelectorPart::default();
                    }
                    pending_combinator = Some(Combinator::Child);
                }
                // Adjacent sibling
                '+' => {
                    while chars
                        .peek()
                        .map(|c| c.is_whitespace())
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                    if !current_part.is_empty() {
                        parts.push((
                            current_part,
                            Some(Combinator::AdjacentSibling),
                        ));
                        current_part = SelectorPart::default();
                    }
                    pending_combinator = Some(Combinator::AdjacentSibling);
                }
                // General sibling
                '~' => {
                    while chars
                        .peek()
                        .map(|c| c.is_whitespace())
                        .unwrap_or(false)
                    {
                        chars.next();
                    }
                    if !current_part.is_empty() {
                        parts.push((
                            current_part,
                            Some(Combinator::GeneralSibling),
                        ));
                        current_part = SelectorPart::default();
                    }
                    pending_combinator = Some(Combinator::GeneralSibling);
                }
                // Class selector
                '.' => {
                    let class_name = collect_identifier(&mut chars);
                    if !class_name.is_empty() {
                        current_part.classes.push(class_name);
                    }
                }
                // ID selector
                '#' => {
                    let id = collect_identifier(&mut chars);
                    if !id.is_empty() {
                        current_part.id = Some(id);
                    }
                }
                // Attribute selector
                '[' => {
                    if let Some(attr) = parse_attribute_selector(&mut chars) {
                        current_part.attributes.push(attr);
                    }
                }
                // Universal selector (skip)
                '*' => {}
                // Pseudo-class/element (skip for now - not needed for Tailwind)
                ':' => {
                    // Skip the pseudo-class/element
                    if chars.peek() == Some(&':') {
                        chars.next(); // Skip second ':'
                    }
                    collect_identifier(&mut chars);
                    // Handle functional pseudo-classes like :not(...)
                    if chars.peek() == Some(&'(') {
                        let mut depth = 1;
                        chars.next();
                        while depth > 0 {
                            match chars.next() {
                                Some('(') => depth += 1,
                                Some(')') => depth -= 1,
                                None => break,
                                _ => {}
                            }
                        }
                    }
                }
                // Element name
                c if c.is_alphanumeric() || c == '_' || c == '-' => {
                    let mut name = String::from(c);
                    while let Some(&next) = chars.peek().filter(|c| {
                        c.is_alphanumeric() || **c == '_' || **c == '-'
                    }) {
                        name.push(next);
                        chars.next();
                    }
                    current_part.element = Some(name.to_lowercase());
                }
                _ => {}
            }
        }

        // Add final part
        if !current_part.is_empty() {
            parts.push((current_part, None));
        }

        // Reverse to get right-to-left order (key selector first)
        parts.reverse();

        SelectorSequence { parts }
    }

    /// Collect an identifier (class name, ID, element name, etc.).
    fn collect_identifier(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> String {
        let mut result = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                result.push(c);
                chars.next();
            } else {
                break;
            }
        }
        result
    }

    /// Parse an attribute selector [name=value].
    fn parse_attribute_selector(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Option<AttributeSelector> {
        let mut name = String::new();
        let mut operator = AttributeOperator::Exists;
        let mut value = String::new();

        // Collect attribute name
        while let Some(&c) = chars.peek() {
            if c == ']'
                || c == '='
                || c == '~'
                || c == '|'
                || c == '^'
                || c == '$'
                || c == '*'
            {
                break;
            }
            name.push(c);
            chars.next();
        }
        name = name.trim().to_string();

        // Check for operator
        if let Some(&op_char) = chars.peek() {
            match op_char {
                '=' => {
                    chars.next();
                    operator = AttributeOperator::Equals;
                }
                '~' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        operator = AttributeOperator::ContainsWord;
                    }
                }
                '|' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        operator = AttributeOperator::DashMatch;
                    }
                }
                '^' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        operator = AttributeOperator::StartsWith;
                    }
                }
                '$' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        operator = AttributeOperator::EndsWith;
                    }
                }
                '*' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        operator = AttributeOperator::Contains;
                    }
                }
                ']' => {}
                _ => {}
            }
        }

        // Collect value (if any)
        if operator != AttributeOperator::Exists {
            // Skip leading whitespace
            while chars.peek() == Some(&' ') {
                chars.next();
            }
            // Check for quoted value
            let quote = match chars.peek() {
                Some(&'"') | Some(&'\'') => {
                    let q = *chars.peek().unwrap();
                    chars.next();
                    Some(q)
                }
                _ => None,
            };

            // Collect value
            while let Some(&c) = chars.peek() {
                if let Some(q) = quote {
                    if c == q {
                        chars.next();
                        break;
                    }
                } else if c == ']' {
                    break;
                }
                value.push(c);
                chars.next();
            }
        }

        // Skip to closing bracket
        while chars.peek().is_some() && chars.peek() != Some(&']') {
            chars.next();
        }
        if chars.peek() == Some(&']') {
            chars.next();
        }

        if name.is_empty() {
            None
        } else {
            Some(AttributeSelector {
                name,
                operator,
                value,
            })
        }
    }

    impl SelectorPart {
        /// Check if this part is empty (no selectors).
        fn is_empty(&self) -> bool {
            self.element.is_none()
                && self.id.is_none()
                && self.classes.is_empty()
                && self.attributes.is_empty()
        }
    }

    // ========================================================================
    // Selector Matching
    // ========================================================================

    /// Check if an element matches a CSS selector string.
    pub fn matches_selector(element: &super::Element, selector: &str) -> bool {
        let parsed = parse_selector(selector);
        matches_parsed_selector(element, &parsed)
    }

    /// Check if an element matches a parsed selector.
    fn matches_parsed_selector(
        element: &super::Element,
        selector: &ParsedSelector,
    ) -> bool {
        // Match if ANY group matches (comma = OR)
        selector
            .groups
            .iter()
            .any(|group| matches_selector_sequence(element, group))
    }

    /// Check if an element matches a selector sequence.
    fn matches_selector_sequence(
        element: &super::Element,
        sequence: &SelectorSequence,
    ) -> bool {
        if sequence.parts.is_empty() {
            return false;
        }

        // First part is the key selector (rightmost in CSS)
        let (key_part, combinator) = &sequence.parts[0];
        if !matches_selector_part(element, key_part) {
            return false;
        }

        // If there's only one part, we're done
        if sequence.parts.len() == 1 {
            return true;
        }

        // Process remaining parts with their combinators
        let mut current_element = element.clone();
        for i in 1..sequence.parts.len() {
            let (part, _next_combinator) = &sequence.parts[i];
            let combinator = sequence.parts[i - 1].1;

            match combinator {
                Some(Combinator::Child) => {
                    // Must match direct parent
                    if let Some(parent) = get_parent_element(&current_element) {
                        if !matches_selector_part(&parent, part) {
                            return false;
                        }
                        current_element = parent;
                    } else {
                        return false;
                    }
                }
                Some(Combinator::Descendant) => {
                    // Must match some ancestor
                    let mut found = false;
                    let mut ancestor = get_parent_element(&current_element);
                    while let Some(anc) = ancestor {
                        if matches_selector_part(&anc, part) {
                            current_element = anc;
                            found = true;
                            break;
                        }
                        ancestor = get_parent_element(&anc);
                    }
                    if !found {
                        return false;
                    }
                }
                Some(Combinator::AdjacentSibling) => {
                    // Must match immediately preceding sibling
                    if let Some(sibling) =
                        get_prev_sibling_element(&current_element)
                    {
                        if !matches_selector_part(&sibling, part) {
                            return false;
                        }
                        current_element = sibling;
                    } else {
                        return false;
                    }
                }
                Some(Combinator::GeneralSibling) => {
                    // Must match some preceding sibling
                    let mut found = false;
                    let mut sibling =
                        get_prev_sibling_element(&current_element);
                    while let Some(sib) = sibling {
                        if matches_selector_part(&sib, part) {
                            current_element = sib;
                            found = true;
                            break;
                        }
                        sibling = get_prev_sibling_element(&sib);
                    }
                    if !found {
                        return false;
                    }
                }
                None => {
                    // No combinator means this is the last part - shouldn't happen
                    // since we handle single-part case above
                }
            }
        }

        true
    }

    /// Check if an element matches a single selector part.
    fn matches_selector_part(
        element: &super::Element,
        part: &SelectorPart,
    ) -> bool {
        super::Document::with_node(element.0 .0, |node| {
            if let super::NodeType::Element {
                tag,
                attrs,
                classes,
                ..
            } = &node.ty
            {
                // Check element type
                if let Some(ref expected_tag) = part.element {
                    if tag.to_lowercase() != *expected_tag {
                        return false;
                    }
                }

                // Check ID
                if let Some(ref expected_id) = part.id {
                    let actual_id =
                        attrs.get("id").map(|s| s.as_str()).unwrap_or("");
                    if actual_id != expected_id {
                        return false;
                    }
                }

                // Check all classes
                for expected_class in &part.classes {
                    if !classes.contains(expected_class) {
                        return false;
                    }
                }

                // Check attributes
                for attr_sel in &part.attributes {
                    let attr_value = attrs.get(&attr_sel.name);
                    match attr_sel.operator {
                        AttributeOperator::Exists => {
                            if attr_value.is_none() {
                                return false;
                            }
                        }
                        AttributeOperator::Equals => {
                            if attr_value.map(|v| v.as_str())
                                != Some(&attr_sel.value[..])
                            {
                                return false;
                            }
                        }
                        AttributeOperator::Contains => {
                            if !attr_value
                                .map(|v| v.contains(&attr_sel.value))
                                .unwrap_or(false)
                            {
                                return false;
                            }
                        }
                        AttributeOperator::StartsWith => {
                            if !attr_value
                                .map(|v| v.starts_with(&attr_sel.value))
                                .unwrap_or(false)
                            {
                                return false;
                            }
                        }
                        AttributeOperator::EndsWith => {
                            if !attr_value
                                .map(|v| v.ends_with(&attr_sel.value))
                                .unwrap_or(false)
                            {
                                return false;
                            }
                        }
                        AttributeOperator::ContainsWord => {
                            if !attr_value
                                .map(|v| {
                                    v.split_whitespace()
                                        .any(|w| w == attr_sel.value)
                                })
                                .unwrap_or(false)
                            {
                                return false;
                            }
                        }
                        AttributeOperator::DashMatch => {
                            if !attr_value
                                .map(|v| {
                                    v == &attr_sel.value
                                        || v.starts_with(&format!(
                                            "{}-",
                                            attr_sel.value
                                        ))
                                })
                                .unwrap_or(false)
                            {
                                return false;
                            }
                        }
                    }
                }

                true
            } else {
                false
            }
        })
        .unwrap_or(false)
    }

    /// Get the parent element of an element.
    fn get_parent_element(element: &super::Element) -> Option<super::Element> {
        super::Document::with_node(element.0 .0, |node| {
            node.parent.and_then(|parent_id| {
                super::Document::with_node(parent_id, |parent_node| {
                    if matches!(parent_node.ty, super::NodeType::Element { .. })
                    {
                        Some(super::Element(super::Node(parent_id)))
                    } else {
                        None
                    }
                })
                .flatten()
            })
        })
        .flatten()
    }

    /// Get the previous sibling element of an element.
    fn get_prev_sibling_element(
        element: &super::Element,
    ) -> Option<super::Element> {
        // Get parent, then find this element in parent's children
        let parent = get_parent_element(element)?;

        super::Document::with_node(parent.0 .0, |parent_node| {
            if let super::NodeType::Element { children, .. } = &parent_node.ty {
                // Find index of current element
                let index =
                    children.iter().position(|c| c.0 == element.0 .0)?;

                // Look backwards for previous element sibling
                for i in (0..index).rev() {
                    let sibling_id = children[i].0;
                    if super::Document::with_node(sibling_id, |node| {
                        matches!(node.ty, super::NodeType::Element { .. })
                    })
                    .unwrap_or(false)
                    {
                        return Some(super::Element(super::Node(sibling_id)));
                    }
                }
                None
            } else {
                None
            }
        })
        .flatten()
    }

    // ========================================================================
    // Cascade Engine
    // ========================================================================

    /// A matching rule with its cascade priority (owned data).
    #[derive(Clone, Debug)]
    struct MatchingRule {
        /// Specificity as (a, b, c) tuple.
        specificity: (u32, u32, u32),
        /// Source order for cascade tie-breaking.
        source_order: usize,
        /// Properties from this rule.
        properties: BTreeMap<String, CssPropertyValue>,
    }

    /// Get a computed style property for an element.
    ///
    /// This applies the CSS cascade to determine the final value:
    /// 1. Find all matching rules
    /// 2. Sort by specificity and source order
    /// 3. Apply !important overrides
    /// 4. Apply inline styles (highest priority)
    pub fn get_computed_style(
        element: &super::Element,
        property: &str,
    ) -> Option<String> {
        let mut computed: Option<String> = None;
        let mut computed_important = false;

        // Find and apply matching rules
        STYLESHEETS.with(|sheets| {
            // Collect matching rules with their cascade priority
            let mut matching: Vec<MatchingRule> = Vec::new();

            for sheet in sheets.borrow().iter() {
                for rule in &sheet.rules {
                    let parsed = parse_selector(&rule.selector_string);
                    if matches_parsed_selector(element, &parsed) {
                        matching.push(MatchingRule {
                            specificity: rule.specificity,
                            source_order: rule.source_order,
                            properties: rule.properties.clone(),
                        });
                    }
                }
            }

            // Sort by cascade order: specificity, then source order
            matching.sort_by_key(|m| (m.specificity, m.source_order));

            // Apply rules in cascade order
            for matching_rule in &matching {
                if let Some(prop) = matching_rule.properties.get(property) {
                    // Apply if:
                    // - No value yet, OR
                    // - New value is !important and old isn't, OR
                    // - Same importance level (later wins due to sort order)
                    if computed.is_none()
                        || (prop.important && !computed_important)
                        || (prop.important == computed_important)
                    {
                        computed = Some(prop.value.clone());
                        computed_important = prop.important;
                    }
                }
            }
        });

        // Inline styles have highest priority (except for !important in stylesheets)
        // For simplicity, inline styles always win
        super::Document::with_node(element.0 .0, |node| {
            if let super::NodeType::Element { inline_styles, .. } = &node.ty {
                if let Some(inline_value) = inline_styles.get(property) {
                    computed = Some(inline_value.clone());
                }
            }
        });

        computed
    }

    /// Get all computed styles for an element as a sorted map.
    ///
    /// Returns a BTreeMap for deterministic ordering (ideal for snapshots).
    /// This is the primary API for snapshot testing - capture the result
    /// with `insta::assert_debug_snapshot!()`.
    pub fn get_all_computed_styles(
        element: &super::Element,
    ) -> BTreeMap<String, String> {
        let mut styles = BTreeMap::new();
        let mut important_properties: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // Find and apply matching rules
        STYLESHEETS.with(|sheets| {
            // Collect matching rules with their cascade priority
            let mut matching: Vec<MatchingRule> = Vec::new();

            for sheet in sheets.borrow().iter() {
                for rule in &sheet.rules {
                    let parsed = parse_selector(&rule.selector_string);
                    if matches_parsed_selector(element, &parsed) {
                        matching.push(MatchingRule {
                            specificity: rule.specificity,
                            source_order: rule.source_order,
                            properties: rule.properties.clone(),
                        });
                    }
                }
            }

            // Sort by cascade order: specificity, then source order
            matching.sort_by_key(|m| (m.specificity, m.source_order));

            // Apply rules in cascade order
            for matching_rule in &matching {
                for (name, prop) in &matching_rule.properties {
                    // Skip if we have an !important value and this isn't !important
                    if important_properties.contains(name) && !prop.important {
                        continue;
                    }
                    styles.insert(name.clone(), prop.value.clone());
                    if prop.important {
                        important_properties.insert(name.clone());
                    }
                }
            }
        });

        // Apply inline styles (highest priority)
        super::Document::with_node(element.0 .0, |node| {
            if let super::NodeType::Element { inline_styles, .. } = &node.ty {
                for (name, value) in inline_styles {
                    styles.insert(name.clone(), value.clone());
                }
            }
        });

        // Resolve CSS variable references
        let resolved: BTreeMap<String, String> = styles
            .into_iter()
            .map(|(name, value)| {
                let resolved_value = resolve_css_variables(&value);
                (name, resolved_value)
            })
            .collect();

        resolved
    }

    /// Look up a CSS variable defined on :root or :host.
    fn lookup_root_variable(var_name: &str) -> Option<String> {
        STYLESHEETS.with(|sheets| {
            for sheet in sheets.borrow().iter() {
                for rule in &sheet.rules {
                    // Match :root, :host, or combined selectors
                    let sel = &rule.selector_string;
                    if sel == ":root"
                        || sel == ":host"
                        || sel.contains(":root")
                        || sel.contains(":host")
                    {
                        let prop_name = format!("--{var_name}");
                        if let Some(prop) = rule.properties.get(&prop_name) {
                            return Some(prop.value.clone());
                        }
                    }
                }
            }
            None
        })
    }

    /// Resolve CSS variable references to their actual values.
    ///
    /// Handles:
    /// - `var(--name)` - looks up --name in :root rules
    /// - `var(--name, fallback)` - uses fallback if --name not found
    /// - Nested variables up to 10 levels deep
    fn resolve_css_variables(value: &str) -> String {
        let mut result = value.to_string();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10; // Prevent infinite loops from circular refs

        // Pattern: var(--name) or var(--name, fallback)
        // Note: fallback can contain nested var() calls
        while result.contains("var(--") && iterations < MAX_ITERATIONS {
            let mut new_result = String::new();
            let mut chars = result.chars().peekable();
            let mut modified = false;

            while let Some(c) = chars.next() {
                // Look for "var("
                if c == 'v' && chars.peek() == Some(&'a') {
                    let rest: String = chars.clone().take(3).collect();
                    if rest == "ar(" {
                        // Consume "ar("
                        chars.next(); // a
                        chars.next(); // r
                        chars.next(); // (

                        // Check for "--"
                        if chars.peek() == Some(&'-') {
                            chars.next(); // first -
                            if chars.peek() == Some(&'-') {
                                chars.next(); // second -

                                // Read variable name (alphanumeric and hyphens)
                                let mut var_name = String::new();
                                while let Some(&ch) = chars.peek() {
                                    if ch.is_alphanumeric()
                                        || ch == '-'
                                        || ch == '_'
                                    {
                                        var_name.push(chars.next().unwrap());
                                    } else {
                                        break;
                                    }
                                }

                                // Check for fallback (comma followed by value)
                                let mut fallback = None;
                                if chars.peek() == Some(&',') {
                                    chars.next(); // consume comma
                                                  // Skip whitespace
                                    while chars.peek() == Some(&' ') {
                                        chars.next();
                                    }
                                    // Read fallback until matching closing paren
                                    let mut fb = String::new();
                                    let mut paren_depth = 1;
                                    while let Some(&ch) = chars.peek() {
                                        if ch == '(' {
                                            paren_depth += 1;
                                            fb.push(chars.next().unwrap());
                                        } else if ch == ')' {
                                            paren_depth -= 1;
                                            if paren_depth == 0 {
                                                break;
                                            }
                                            fb.push(chars.next().unwrap());
                                        } else {
                                            fb.push(chars.next().unwrap());
                                        }
                                    }
                                    fallback = Some(fb);
                                }

                                // Consume closing paren
                                if chars.peek() == Some(&')') {
                                    chars.next();
                                }

                                // Look up variable and substitute
                                if let Some(var_value) =
                                    lookup_root_variable(&var_name)
                                {
                                    new_result.push_str(&var_value);
                                    modified = true;
                                } else if let Some(fb) = fallback {
                                    new_result.push_str(&fb);
                                    modified = true;
                                } else {
                                    // Keep original if not found and no fallback
                                    new_result.push_str(&format!(
                                        "var(--{var_name})"
                                    ));
                                }
                                continue;
                            } else {
                                // Wasn't "--", put back what we consumed
                                new_result.push_str("var(-");
                                continue;
                            }
                        } else {
                            // var( but not var(--, treat as regular text
                            new_result.push_str("var(");
                            continue;
                        }
                    }
                }
                new_result.push(c);
            }

            if modified {
                result = new_result;
            } else {
                break;
            }
            iterations += 1;
        }

        result
    }
}

/// The type of mock DOM node.
#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    /// A text node.
    Text(String),
    /// An element.
    Element {
        /// The HTML tag name.
        tag: Cow<'static, str>,
        /// The attributes.
        attrs: IndexMap<String, String>,
        /// The element's children.
        children: Vec<Node>,
        /// CSS classes (preserves insertion order).
        classes: Vec<String>,
        /// Inline styles (preserves insertion order).
        inline_styles: IndexMap<String, String>,
    },
    /// A placeholder.
    Placeholder,
}

impl MockDom {
    /// Interns a string (no-op for mock DOM).
    pub fn intern(text: &str) -> &str {
        text
    }

    /// Creates a new element with the given tag name and optional namespace.
    pub fn create_element(tag: &str, namespace: Option<&str>) -> Element {
        // Namespace is ignored in mock DOM - it's mainly for SVG/MathML
        let _ = namespace;
        document().create_element(tag)
    }

    /// Creates a new text node.
    pub fn create_text_node(data: &str) -> Text {
        document().create_text_node(data)
    }

    /// Creates a new placeholder node.
    pub fn create_placeholder() -> Placeholder {
        document().create_placeholder()
    }

    /// Sets the text content of a text node.
    pub fn set_text(node: &Text, text: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Text(ref mut content) = node.ty {
                *content = text.to_string();
            }
        });
    }

    /// Sets an attribute on an element.
    ///
    /// Special handling for "class" attribute: parses the value into individual
    /// class names and stores them in the `classes` vector for proper snapshot output.
    pub fn set_attribute(node: &Element, name: &str, value: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Element {
                ref mut attrs,
                ref mut classes,
                ..
            } = node.ty
            {
                if name == "class" {
                    // Parse class string into individual class names
                    classes.clear();
                    for class in value.split_whitespace() {
                        if !class.is_empty() {
                            classes.push(class.to_string());
                        }
                    }
                } else {
                    attrs.insert(name.to_string(), value.to_string());
                }
            }
        });
    }

    /// Removes an attribute from an element.
    pub fn remove_attribute(node: &Element, name: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Element { ref mut attrs, .. } = node.ty {
                attrs.shift_remove(name);
            }
        });
    }

    /// Tries to insert a node, returning success status.
    pub fn try_insert_node(
        parent: &Element,
        new_child: &Node,
        anchor: Option<&Node>,
    ) -> bool {
        Self::insert_node(parent, new_child, anchor);
        true
    }

    /// Inserts a node before an anchor, or appends if anchor is None.
    pub fn insert_node(
        parent: &Element,
        new_child: &Node,
        anchor: Option<&Node>,
    ) {
        debug_assert!(&parent.0 != new_child);
        // remove if already mounted
        if let Some(old_parent) = Self::get_parent(new_child) {
            let old_parent = Element(old_parent);
            Self::remove_node(&old_parent, new_child);
        }
        // mount on new parent
        Document::with_node_mut(parent.0 .0, |parent_data| {
            if let NodeType::Element {
                ref mut children, ..
            } = parent_data.ty
            {
                match anchor {
                    None => children.push(new_child.clone()),
                    Some(anchor) => {
                        let anchor_pos = children
                            .iter()
                            .position(|item| item.0 == anchor.0)
                            .expect("anchor is not a child of the parent");
                        children.insert(anchor_pos, new_child.clone());
                    }
                }
            } else {
                panic!("parent is not an element");
            }
        });
        // set parent on child node
        Document::with_node_mut(new_child.0, |node| {
            node.parent = Some(parent.0 .0)
        });
    }

    /// Removes a child node from a parent.
    pub fn remove_node(parent: &Element, child: &Node) -> Option<Node> {
        let removed = Document::with_node_mut(parent.0 .0, |parent_data| {
            if let NodeType::Element {
                ref mut children, ..
            } = parent_data.ty
            {
                children
                    .iter()
                    .position(|item| item.0 == child.0)
                    .map(|pos| children.remove(pos))
            } else {
                None
            }
        })
        .flatten()?;
        Document::with_node_mut(removed.0, |node| {
            node.parent = None;
        });
        Some(removed)
    }

    /// Removes a node from the DOM.
    pub fn remove(node: &Node) {
        if let Some(parent) = Self::get_parent(node) {
            let parent = Element(parent);
            Self::remove_node(&parent, node);
        }
    }

    /// Gets the parent of a node.
    pub fn get_parent(node: &Node) -> Option<Node> {
        Document::with_node(node.0, |node| node.parent)
            .flatten()
            .map(Node)
    }

    /// Gets the first child of a node.
    pub fn first_child(node: &Node) -> Option<Node> {
        Document::with_node(node.0, |node| match &node.ty {
            NodeType::Text(_) => None,
            NodeType::Element { children, .. } => children.first().cloned(),
            NodeType::Placeholder => None,
        })
        .flatten()
    }

    /// Gets the next sibling of a node.
    pub fn next_sibling(node: &Node) -> Option<Node> {
        let node_id = node.0;
        Document::with_node(node_id, |node| {
            node.parent.and_then(|parent| {
                Document::with_node(parent, |parent| match &parent.ty {
                    NodeType::Element { children, .. } => {
                        let this = children
                            .iter()
                            .position(|check| check == &Node(node_id))?;
                        children.get(this + 1).cloned()
                    }
                    _ => panic!(
                        "Called next_sibling with parent as a node that's not \
                         an Element."
                    ),
                })
            })
        })
        .flatten()
        .flatten()
    }

    /// Logs a node (for debugging).
    pub fn log_node(node: &Node) {
        eprintln!("{node:?}");
    }

    /// Clears all children from an element.
    pub fn clear_children(parent: &Element) {
        let prev_children =
            Document::with_node_mut(parent.0 .0, |node| match node.ty {
                NodeType::Element {
                    ref mut children, ..
                } => std::mem::take(children),
                _ => panic!("Called clear_children on a non-Element node."),
            })
            .unwrap_or_default();
        for child in prev_children {
            Document::with_node_mut(child.0, |node| {
                node.parent = None;
            });
        }
    }

    /// Sets the inner HTML of an element.
    pub fn set_inner_html(el: &Element, html: &str) {
        // Clear children and add raw HTML as text (simplified)
        Document::with_node_mut(el.0 .0, |node| {
            if let NodeType::Element {
                ref mut children, ..
            } = node.ty
            {
                children.clear();
            }
        });
        // For simplicity, we just store the HTML as a text node
        let text = document().create_text_node(html);
        Self::insert_node(el, text.as_ref(), None);
    }

    /// Gets a template element.
    pub fn get_template<V>() -> TemplateElement
    where
        V: crate::view::ToTemplate + 'static,
    {
        TemplateElement
    }

    /// Clones a template element.
    pub fn clone_template(_tpl: &TemplateElement) -> Element {
        document().create_element("div")
    }

    /// Creates an element from HTML.
    pub fn create_element_from_html(
        _html: std::borrow::Cow<'static, str>,
    ) -> Element {
        document().create_element("div")
    }

    /// Sets a JavaScript object property on a DOM element.
    /// No-op for mock DOM - properties are JS-specific.
    pub fn set_property(
        _el: &Element,
        _key: &str,
        _value: &wasm_bindgen::JsValue,
    ) {
        // No-op for mock DOM
    }

    /// Adds an event listener to an element.
    /// Stores the callback for later testing with dispatch_event.
    pub fn add_event_listener(
        el: &Element,
        name: &str,
        cb: Box<dyn FnMut(Event)>,
    ) -> super::RemoveEventHandler<Element> {
        Self::add_event_listener_internal(el, name, cb, false)
    }

    /// Adds an event listener to an element with use_capture option.
    /// Stores the callback for later testing with dispatch_event.
    pub fn add_event_listener_use_capture(
        el: &Element,
        name: &str,
        cb: Box<dyn FnMut(Event)>,
    ) -> super::RemoveEventHandler<Element> {
        Self::add_event_listener_internal(el, name, cb, true)
    }

    /// Adds a delegated event listener to an element.
    /// For mock_dom, treated the same as a regular listener.
    pub fn add_event_listener_delegated(
        el: &Element,
        name: std::borrow::Cow<'static, str>,
        _delegation_key: std::borrow::Cow<'static, str>,
        cb: Box<dyn FnMut(Event)>,
    ) -> super::RemoveEventHandler<Element> {
        Self::add_event_listener_internal(el, &name, cb, false)
    }

    fn add_event_listener_internal(
        el: &Element,
        name: &str,
        cb: Box<dyn FnMut(Event)>,
        use_capture: bool,
    ) -> super::RemoveEventHandler<Element> {
        let node_id = el.0 .0;
        let callback: EventCallback = Rc::new(RefCell::new(cb));
        let listener = StoredListener {
            event_name: name.to_string(),
            callback,
            use_capture,
        };

        EVENT_LISTENERS.with(|listeners| {
            listeners
                .borrow_mut()
                .entry(node_id)
                .or_default()
                .push(listener);
        });

        let name = name.to_string();
        super::RemoveEventHandler::new(move || {
            EVENT_LISTENERS.with(|listeners| {
                if let Some(list) = listeners.borrow_mut().get_mut(&node_id) {
                    list.retain(|l| l.event_name != name);
                }
            });
        })
    }

    /// Return the event target element.
    pub fn event_target<T>(ev: &Event) -> T
    where
        T: super::CastFrom<Element>,
    {
        let node_id = ev.target.expect("event has no target");
        let element = Element(Node(node_id));
        T::cast_from(element).expect("incorrect element type for event target")
    }

    /// Gets the class list for an element.
    pub fn class_list(el: &Element) -> ClassList {
        ClassList {
            element_id: el.0 .0,
        }
    }

    /// Adds a class to the class list.
    pub fn add_class(class_list: &ClassList, name: &str) {
        Document::with_node_mut(class_list.element_id, |node| {
            if let NodeType::Element {
                ref mut classes, ..
            } = node.ty
            {
                // Only add if not already present
                if !classes.contains(&name.to_string()) {
                    classes.push(name.to_string());
                }
            }
        });
    }

    /// Removes a class from the class list.
    pub fn remove_class(class_list: &ClassList, name: &str) {
        Document::with_node_mut(class_list.element_id, |node| {
            if let NodeType::Element {
                ref mut classes, ..
            } = node.ty
            {
                classes.retain(|c| c != name);
            }
        });
    }

    /// Toggles a class on the class list. Returns true if the class is now present.
    pub fn toggle_class(class_list: &ClassList, name: &str) -> bool {
        Document::with_node_mut(class_list.element_id, |node| {
            if let NodeType::Element {
                ref mut classes, ..
            } = node.ty
            {
                if let Some(pos) = classes.iter().position(|c| c == name) {
                    classes.remove(pos);
                    false
                } else {
                    classes.push(name.to_string());
                    true
                }
            } else {
                false
            }
        })
        .unwrap_or(false)
    }

    /// Checks if the class list contains the given class.
    pub fn contains_class(class_list: &ClassList, name: &str) -> bool {
        Document::with_node(class_list.element_id, |node| {
            if let NodeType::Element { ref classes, .. } = node.ty {
                classes.contains(&name.to_string())
            } else {
                false
            }
        })
        .unwrap_or(false)
    }

    /// Gets the style declaration for an element.
    pub fn style(el: &Element) -> CssStyleDeclaration {
        CssStyleDeclaration {
            element_id: el.0 .0,
        }
    }

    /// Sets a CSS property on the inline style.
    pub fn set_css_property(
        style: &CssStyleDeclaration,
        name: &str,
        value: &str,
    ) {
        Document::with_node_mut(style.element_id, |node| {
            if let NodeType::Element {
                ref mut inline_styles,
                ..
            } = node.ty
            {
                inline_styles.insert(name.to_string(), value.to_string());
            }
        });
    }

    /// Removes a CSS property from the inline style.
    pub fn remove_css_property(style: &CssStyleDeclaration, name: &str) {
        Document::with_node_mut(style.element_id, |node| {
            if let NodeType::Element {
                ref mut inline_styles,
                ..
            } = node.ty
            {
                inline_styles.shift_remove(name);
            }
        });
    }

    /// Gets the value of a CSS property from the inline style.
    pub fn get_css_property(
        style: &CssStyleDeclaration,
        name: &str,
    ) -> Option<String> {
        Document::with_node(style.element_id, |node| {
            if let NodeType::Element {
                ref inline_styles, ..
            } = node.ty
            {
                inline_styles.get(name).cloned()
            } else {
                None
            }
        })
        .flatten()
    }

    /// Sets a property or value (used for form element values).
    /// No-op for mock DOM.
    pub fn set_property_or_value(
        _el: &Element,
        _key: &str,
        _value: &wasm_bindgen::JsValue,
    ) {
        // No-op for mock DOM
    }

    /// Mounts a child before a marker node.
    pub fn mount_before<M>(new_child: &mut M, before: &Node)
    where
        M: crate::view::Mountable,
    {
        let parent = Element::cast_from(
            Self::get_parent(before).expect("could not find parent element"),
        )
        .expect("placeholder parent should be Element");
        new_child.mount(&parent, Some(before));
    }

    /// Tries to mount a child before a marker node.
    pub fn try_mount_before<M>(new_child: &mut M, before: &Node) -> bool
    where
        M: crate::view::Mountable,
    {
        if let Some(parent) =
            Self::get_parent(before).and_then(Element::cast_from)
        {
            new_child.mount(&parent, Some(before));
            true
        } else {
            false
        }
    }

    /// Creates an element from SVG HTML.
    pub fn create_svg_element_from_html(
        _html: std::borrow::Cow<'static, str>,
    ) -> Element {
        document().create_element("svg")
    }
}

impl CastFrom<Node> for Text {
    fn cast_from(source: Node) -> Option<Self> {
        Document::with_node(source.0, |node| {
            matches!(node.ty, NodeType::Text(_))
        })
        .and_then(|matches| matches.then_some(Text(Node(source.0))))
    }
}

impl CastFrom<Node> for Element {
    fn cast_from(source: Node) -> Option<Self> {
        Document::with_node(source.0, |node| {
            matches!(node.ty, NodeType::Element { .. })
        })
        .and_then(|matches| matches.then_some(Element(Node(source.0))))
    }
}

impl CastFrom<Node> for Placeholder {
    fn cast_from(source: Node) -> Option<Self> {
        Document::with_node(source.0, |node| {
            matches!(node.ty, NodeType::Placeholder)
        })
        .and_then(|matches| matches.then_some(Placeholder(Node(source.0))))
    }
}

// Renderer and DomRenderer trait implementations

impl super::Renderer for MockDom {
    type Node = Node;
    type Element = Element;
    type Text = Text;
    type Placeholder = Placeholder;

    fn intern(text: &str) -> &str {
        MockDom::intern(text)
    }

    fn create_text_node(text: &str) -> Self::Text {
        MockDom::create_text_node(text)
    }

    fn create_placeholder() -> Self::Placeholder {
        MockDom::create_placeholder()
    }

    fn set_text(node: &Self::Text, text: &str) {
        MockDom::set_text(node, text)
    }

    fn set_attribute(node: &Self::Element, name: &str, value: &str) {
        MockDom::set_attribute(node, name, value)
    }

    fn remove_attribute(node: &Self::Element, name: &str) {
        MockDom::remove_attribute(node, name)
    }

    fn insert_node(
        parent: &Self::Element,
        new_child: &Self::Node,
        marker: Option<&Self::Node>,
    ) {
        MockDom::insert_node(parent, new_child, marker)
    }

    fn remove_node(
        parent: &Self::Element,
        child: &Self::Node,
    ) -> Option<Self::Node> {
        MockDom::remove_node(parent, child)
    }

    fn clear_children(parent: &Self::Element) {
        MockDom::clear_children(parent)
    }

    fn remove(node: &Self::Node) {
        MockDom::remove(node)
    }

    fn get_parent(node: &Self::Node) -> Option<Self::Node> {
        MockDom::get_parent(node)
    }

    fn first_child(node: &Self::Node) -> Option<Self::Node> {
        MockDom::first_child(node)
    }

    fn next_sibling(node: &Self::Node) -> Option<Self::Node> {
        MockDom::next_sibling(node)
    }

    fn log_node(node: &Self::Node) {
        MockDom::log_node(node)
    }
}

impl super::DomRenderer for MockDom {
    type Event = Event;
    type ClassList = ClassList;
    type CssStyleDeclaration = CssStyleDeclaration;
    type TemplateElement = TemplateElement;

    fn set_property(
        _el: &Self::Element,
        _key: &str,
        _value: &wasm_bindgen::JsValue,
    ) {
        // No-op for mock DOM - properties are JS-specific
    }

    fn add_event_listener(
        el: &Self::Element,
        name: &str,
        cb: Box<dyn FnMut(Self::Event)>,
    ) -> super::RemoveEventHandler<Self::Element> {
        MockDom::add_event_listener(el, name, cb)
    }

    fn add_event_listener_delegated(
        el: &Self::Element,
        name: std::borrow::Cow<'static, str>,
        delegation_key: std::borrow::Cow<'static, str>,
        cb: Box<dyn FnMut(Self::Event)>,
    ) -> super::RemoveEventHandler<Self::Element> {
        MockDom::add_event_listener_delegated(el, name, delegation_key, cb)
    }

    fn event_target<T>(ev: &Self::Event) -> T
    where
        T: super::CastFrom<Self::Element>,
    {
        MockDom::event_target(ev)
    }

    fn class_list(el: &Self::Element) -> Self::ClassList {
        MockDom::class_list(el)
    }

    fn add_class(class_list: &Self::ClassList, name: &str) {
        MockDom::add_class(class_list, name)
    }

    fn remove_class(class_list: &Self::ClassList, name: &str) {
        MockDom::remove_class(class_list, name)
    }

    fn style(el: &Self::Element) -> Self::CssStyleDeclaration {
        MockDom::style(el)
    }

    fn set_css_property(
        style: &Self::CssStyleDeclaration,
        name: &str,
        value: &str,
    ) {
        MockDom::set_css_property(style, name, value)
    }

    fn set_inner_html(el: &Self::Element, html: &str) {
        MockDom::set_inner_html(el, html)
    }

    fn get_template<V>() -> Self::TemplateElement
    where
        V: crate::view::ToTemplate + 'static,
    {
        MockDom::get_template::<V>()
    }

    fn clone_template(tpl: &Self::TemplateElement) -> Self::Element {
        MockDom::clone_template(tpl)
    }

    fn create_element_from_html(html: &str) -> Self::Element {
        MockDom::create_element_from_html(std::borrow::Cow::Owned(
            html.to_owned(),
        ))
    }
}

// Mountable implementations for mock DOM types
// These are conditionally compiled only when mock_dom feature is enabled

impl Mountable for Node {
    fn unmount(&mut self) {
        MockDom::remove(self);
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent = MockDom::get_parent(self).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self));
            true
        } else {
            false
        }
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}

impl Mountable for Element {
    fn unmount(&mut self) {
        MockDom::remove(self.as_ref());
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent =
            MockDom::get_parent(self.as_ref()).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self.as_ref()));
            true
        } else {
            false
        }
    }

    fn elements(&self) -> Vec<Element> {
        vec![self.clone()]
    }
}

impl Mountable for Text {
    fn unmount(&mut self) {
        MockDom::remove(self.as_ref());
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent =
            MockDom::get_parent(self.as_ref()).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self.as_ref()));
            true
        } else {
            false
        }
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}

impl Mountable for Placeholder {
    fn unmount(&mut self) {
        MockDom::remove(self.as_ref());
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        MockDom::insert_node(parent, self.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent =
            MockDom::get_parent(self.as_ref()).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self.as_ref()));
            true
        } else {
            false
        }
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}

// ============================================================================
// Mock Event Wrapper Types
// ============================================================================
// These types mirror the web_sys event types and are used when mock_dom is enabled.
// They implement From<MockEvent> and provide the same API as web_sys events.

/// Mock event wrapper types that mirror web_sys event APIs.
#[allow(missing_docs)]
pub mod events {
    use super::{
        Document, MockAnimationData, MockEvent, MockFocusData,
        MockKeyboardData, MockMouseData, MockTouchData, MockTransitionData,
        MockWheelData, Node, NodeType,
    };
    use wasm_bindgen::convert::FromWasmAbi;
    use wasm_bindgen::describe::WasmDescribe;

    // ========== EventTarget ==========
    /// Mock EventTarget - represents the target of an event.
    /// Can be cast to specific element types using dyn_into.
    #[derive(Clone, Debug)]
    pub struct EventTarget(pub(crate) Node);

    impl EventTarget {
        /// Attempt to cast this target to a specific element type.
        /// In mock mode, this succeeds for any element type.
        pub fn dyn_into<T: FromEventTarget>(self) -> Result<T, Self> {
            T::from_event_target(self)
        }

        /// Unchecked cast to a specific type.
        pub fn unchecked_into<T: FromEventTarget>(self) -> T {
            T::from_event_target(self).expect("unchecked_into failed")
        }

        /// Get a reference and attempt to cast.
        pub fn dyn_ref<T: FromEventTarget>(&self) -> Option<T> {
            T::from_event_target(self.clone()).ok()
        }
    }

    /// Trait for types that can be converted from EventTarget in mock mode.
    /// This allows dyn_into to work without conflicting with std TryFrom.
    pub trait FromEventTarget: Sized {
        /// Convert from EventTarget. In mock mode this always succeeds.
        fn from_event_target(target: EventTarget) -> Result<Self, EventTarget>;
    }

    /// Trait for types that can be converted from Element in mock mode.
    /// This allows dyn_ref to work for Element -> HtmlElement conversions.
    pub trait FromElement: Sized {
        /// Convert from Element. In mock mode this always succeeds.
        fn from_element(element: &super::Element) -> Option<Self>;
    }

    // ========== HtmlInputElement ==========
    /// Mock HtmlInputElement
    #[derive(Clone, Debug)]
    pub struct HtmlInputElement(pub(crate) Node);

    impl HtmlInputElement {
        /// Get the input value
        pub fn value(&self) -> String {
            Document::with_node(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &node.ty {
                    attrs.get("value").cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            })
            .unwrap_or_default()
        }

        /// Set the input value
        pub fn set_value(&self, value: &str) {
            Document::with_node_mut(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &mut node.ty {
                    attrs.insert("value".to_string(), value.to_string());
                }
            });
        }

        /// Get files (returns None in mock mode)
        pub fn files(&self) -> Option<FileList> {
            None
        }

        /// Check if this element is checked (for checkboxes/radios)
        pub fn checked(&self) -> bool {
            Document::with_node(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &node.ty {
                    attrs.get("checked").map(|v| v == "true").unwrap_or(false)
                } else {
                    false
                }
            })
            .unwrap_or(false)
        }

        /// Set the checked state
        pub fn set_checked(&self, value: bool) {
            Document::with_node_mut(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &mut node.ty {
                    if value {
                        attrs.insert("checked".to_string(), "true".to_string());
                    } else {
                        attrs.shift_remove("checked");
                    }
                }
            });
        }
    }

    impl FromEventTarget for HtmlInputElement {
        fn from_event_target(target: EventTarget) -> Result<Self, EventTarget> {
            Ok(HtmlInputElement(target.0))
        }
    }

    // ========== HtmlElement ==========
    /// Mock HtmlElement (generic)
    #[derive(Clone, Debug)]
    pub struct HtmlElement(pub(crate) Node);

    impl FromEventTarget for HtmlElement {
        fn from_event_target(target: EventTarget) -> Result<Self, EventTarget> {
            Ok(HtmlElement(target.0))
        }
    }

    impl FromElement for HtmlElement {
        fn from_element(element: &super::Element) -> Option<Self> {
            // In mock mode, any Element can be treated as HtmlElement
            Some(HtmlElement(element.0.clone()))
        }
    }

    impl HtmlElement {
        /// Get the left offset position of the element.
        /// In mock mode, reads from `data-mock-offset-left` attribute or returns 0.
        pub fn offset_left(&self) -> i32 {
            Document::with_node(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &node.ty {
                    attrs
                        .get("data-mock-offset-left")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .unwrap_or(0)
        }

        /// Get the width of the element.
        /// In mock mode, reads from `data-mock-offset-width` attribute or returns 100.
        pub fn offset_width(&self) -> i32 {
            Document::with_node(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &node.ty {
                    attrs
                        .get("data-mock-offset-width")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(100)
                } else {
                    100
                }
            })
            .unwrap_or(100)
        }

        /// Get the top offset position of the element.
        /// In mock mode, reads from `data-mock-offset-top` attribute or returns 0.
        pub fn offset_top(&self) -> i32 {
            Document::with_node(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &node.ty {
                    attrs
                        .get("data-mock-offset-top")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .unwrap_or(0)
        }

        /// Get the height of the element.
        /// In mock mode, reads from `data-mock-offset-height` attribute or returns 100.
        pub fn offset_height(&self) -> i32 {
            Document::with_node(self.0 .0, |node| {
                if let NodeType::Element { attrs, .. } = &node.ty {
                    attrs
                        .get("data-mock-offset-height")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(100)
                } else {
                    100
                }
            })
            .unwrap_or(100)
        }

        /// Focus the element.
        /// In mock mode, this is a no-op that always succeeds.
        pub fn focus(&self) -> Result<(), JsValue> {
            Ok(())
        }
    }

    /// Mock JsValue type for error returns.
    #[derive(Clone, Debug)]
    pub struct JsValue;

    // ========== FileList ==========
    /// Mock FileList (empty in mock mode)
    #[derive(Clone, Debug)]
    pub struct FileList;

    impl FileList {
        /// Returns the number of files (always 0 in mock)
        pub fn length(&self) -> u32 {
            0
        }
        /// Get file by index (always None in mock)
        pub fn item(&self, _index: u32) -> Option<File> {
            None
        }
        /// Get file by index (always None in mock)
        pub fn get(&self, _index: u32) -> Option<File> {
            None
        }
    }

    // ========== File ==========
    /// Mock File
    #[derive(Clone, Debug)]
    pub struct File;

    impl File {
        /// Get the file name (empty in mock)
        pub fn name(&self) -> String {
            String::new()
        }
        /// Get the file size (0 in mock)
        pub fn size(&self) -> f64 {
            0.0
        }
        /// Get the file type (empty in mock)
        pub fn type_(&self) -> String {
            String::new()
        }
    }

    // Helper macro to implement common event methods
    macro_rules! impl_event_common {
        ($name:ident) => {
            impl $name {
                /// Returns the event type.
                pub fn type_(&self) -> String {
                    self.0.event_type.clone()
                }

                /// Prevents the default action.
                pub fn prevent_default(&self) {
                    self.0.default_prevented.set(true);
                }

                /// Stops event propagation.
                pub fn stop_propagation(&self) {
                    self.0.propagation_stopped.set(true);
                }

                /// Stops immediate propagation (prevents other listeners on same element).
                pub fn stop_immediate_propagation(&self) {
                    self.0.propagation_stopped.set(true);
                    self.0.immediate_propagation_stopped.set(true);
                }

                /// Returns the current target (element currently handling the event).
                pub fn current_target(&self) -> Option<super::Element> {
                    self.0
                        .current_target
                        .get()
                        .map(|id| super::Element(super::Node(id)))
                }

                /// Returns the target element (the element that originally dispatched the event).
                pub fn target(&self) -> Option<EventTarget> {
                    self.0.target.map(|id| EventTarget(super::Node(id)))
                }

                /// Returns the event phase (0=None, 1=Capturing, 2=AtTarget, 3=Bubbling).
                pub fn event_phase(&self) -> u16 {
                    self.0.event_phase.get() as u16
                }

                /// Returns whether default was prevented.
                pub fn default_prevented(&self) -> bool {
                    self.0.default_prevented.get()
                }

                /// Returns whether this event bubbles.
                pub fn bubbles(&self) -> bool {
                    self.0.bubbles
                }

                /// Returns whether this event is cancelable.
                pub fn cancelable(&self) -> bool {
                    self.0.cancelable
                }

                /// Returns the event timestamp.
                pub fn time_stamp(&self) -> f64 {
                    self.0.time_stamp
                }

                /// Returns the input value if this event carries input data.
                ///
                /// This is used by mock_dom's `event_target_value` helper to extract
                /// the value from events created via `testing::input(el, value)`.
                pub fn input_value(&self) -> String {
                    self.0
                        .input
                        .as_ref()
                        .and_then(|i| i.data.clone())
                        .unwrap_or_default()
                }
            }

            impl From<MockEvent> for $name {
                fn from(ev: MockEvent) -> Self {
                    $name(ev)
                }
            }

            // Stub WasmDescribe - required by FromWasmAbi
            impl WasmDescribe for $name {
                fn describe() {
                    // Describe as externref/anyref - we never actually use this
                    <wasm_bindgen::JsValue as WasmDescribe>::describe()
                }
            }

            // Stub FromWasmAbi - never actually called in tests
            impl FromWasmAbi for $name {
                type Abi = u32;
                #[inline]
                unsafe fn from_abi(_js: u32) -> Self {
                    panic!(concat!(
                        stringify!($name),
                        "::from_abi should never be called in mock_dom tests"
                    ))
                }
            }
        };
    }

    // Helper macro for mouse event methods
    macro_rules! impl_mouse_methods {
        ($name:ident) => {
            impl $name {
                fn mouse(&self) -> MockMouseData {
                    self.0.mouse.clone().unwrap_or_default()
                }

                pub fn client_x(&self) -> i32 {
                    self.mouse().client_x
                }
                pub fn client_y(&self) -> i32 {
                    self.mouse().client_y
                }
                pub fn page_x(&self) -> f64 {
                    self.mouse().page_x
                }
                pub fn page_y(&self) -> f64 {
                    self.mouse().page_y
                }
                pub fn screen_x(&self) -> i32 {
                    self.mouse().screen_x
                }
                pub fn screen_y(&self) -> i32 {
                    self.mouse().screen_y
                }
                pub fn offset_x(&self) -> f64 {
                    self.mouse().offset_x
                }
                pub fn offset_y(&self) -> f64 {
                    self.mouse().offset_y
                }
                pub fn movement_x(&self) -> i32 {
                    self.mouse().movement_x
                }
                pub fn movement_y(&self) -> i32 {
                    self.mouse().movement_y
                }
                pub fn button(&self) -> i16 {
                    self.mouse().button
                }
                pub fn buttons(&self) -> u16 {
                    self.mouse().buttons
                }
                pub fn alt_key(&self) -> bool {
                    self.mouse().alt_key
                }
                pub fn ctrl_key(&self) -> bool {
                    self.mouse().ctrl_key
                }
                pub fn meta_key(&self) -> bool {
                    self.mouse().meta_key
                }
                pub fn shift_key(&self) -> bool {
                    self.mouse().shift_key
                }
                pub fn get_modifier_state(&self, _key: &str) -> bool {
                    false
                }
            }
        };
    }

    // Helper macro for keyboard event methods
    macro_rules! impl_keyboard_methods {
        ($name:ident) => {
            impl $name {
                fn keyboard(&self) -> MockKeyboardData {
                    self.0.keyboard.clone().unwrap_or_default()
                }

                pub fn key(&self) -> String {
                    self.keyboard().key
                }
                pub fn code(&self) -> String {
                    self.keyboard().code
                }
                pub fn location(&self) -> u32 {
                    self.keyboard().location
                }
                pub fn repeat(&self) -> bool {
                    self.keyboard().repeat
                }
                pub fn alt_key(&self) -> bool {
                    self.keyboard().alt_key
                }
                pub fn ctrl_key(&self) -> bool {
                    self.keyboard().ctrl_key
                }
                pub fn meta_key(&self) -> bool {
                    self.keyboard().meta_key
                }
                pub fn shift_key(&self) -> bool {
                    self.keyboard().shift_key
                }
                pub fn is_composing(&self) -> bool {
                    self.keyboard().is_composing
                }
                pub fn char_code(&self) -> u32 {
                    self.keyboard().char_code
                }
                pub fn key_code(&self) -> u32 {
                    self.keyboard().key_code
                }
                pub fn get_modifier_state(&self, _key: &str) -> bool {
                    false
                }
            }
        };
    }

    // ========== Event (base) ==========
    #[derive(Clone, Debug)]
    pub struct Event(pub(crate) MockEvent);
    impl_event_common!(Event);

    // ========== MouseEvent ==========
    #[derive(Clone, Debug)]
    pub struct MouseEvent(pub(crate) MockEvent);
    impl_event_common!(MouseEvent);
    impl_mouse_methods!(MouseEvent);

    // ========== KeyboardEvent ==========
    #[derive(Clone, Debug)]
    pub struct KeyboardEvent(pub(crate) MockEvent);
    impl_event_common!(KeyboardEvent);
    impl_keyboard_methods!(KeyboardEvent);

    // ========== FocusEvent ==========
    #[derive(Clone, Debug)]
    pub struct FocusEvent(pub(crate) MockEvent);
    impl_event_common!(FocusEvent);
    impl FocusEvent {
        fn focus(&self) -> MockFocusData {
            self.0.focus.clone().unwrap_or_default()
        }
        // related_target would return Element, but we simplify for mock
    }

    // ========== InputEvent ==========
    #[derive(Clone, Debug)]
    pub struct InputEvent(pub(crate) MockEvent);
    impl_event_common!(InputEvent);
    impl InputEvent {
        pub fn data(&self) -> Option<String> {
            self.0.input.as_ref().and_then(|i| i.data.clone())
        }
        pub fn input_type(&self) -> String {
            self.0
                .input
                .as_ref()
                .map(|i| i.input_type.clone())
                .unwrap_or_default()
        }
        pub fn is_composing(&self) -> bool {
            self.0
                .input
                .as_ref()
                .map(|i| i.is_composing)
                .unwrap_or(false)
        }
    }

    // ========== WheelEvent ==========
    #[derive(Clone, Debug)]
    pub struct WheelEvent(pub(crate) MockEvent);
    impl_event_common!(WheelEvent);
    impl_mouse_methods!(WheelEvent);
    impl WheelEvent {
        fn wheel(&self) -> MockWheelData {
            self.0.wheel.clone().unwrap_or_default()
        }
        pub fn delta_x(&self) -> f64 {
            self.wheel().delta_x
        }
        pub fn delta_y(&self) -> f64 {
            self.wheel().delta_y
        }
        pub fn delta_z(&self) -> f64 {
            self.wheel().delta_z
        }
        pub fn delta_mode(&self) -> u32 {
            self.wheel().delta_mode
        }
    }

    // ========== PointerEvent ==========
    #[derive(Clone, Debug)]
    pub struct PointerEvent(pub(crate) MockEvent);
    impl_event_common!(PointerEvent);
    impl_mouse_methods!(PointerEvent);
    impl PointerEvent {
        pub fn pointer_id(&self) -> i32 {
            self.mouse().pointer_id
        }
        pub fn width(&self) -> f64 {
            self.mouse().width
        }
        pub fn height(&self) -> f64 {
            self.mouse().height
        }
        pub fn pressure(&self) -> f32 {
            self.mouse().pressure
        }
        pub fn tilt_x(&self) -> i32 {
            self.mouse().tilt_x
        }
        pub fn tilt_y(&self) -> i32 {
            self.mouse().tilt_y
        }
        pub fn pointer_type(&self) -> String {
            self.mouse().pointer_type.clone()
        }
        pub fn is_primary(&self) -> bool {
            self.mouse().is_primary
        }
    }

    // ========== TouchEvent ==========
    #[derive(Clone, Debug)]
    pub struct TouchEvent(pub(crate) MockEvent);
    impl_event_common!(TouchEvent);
    impl TouchEvent {
        fn touch(&self) -> MockTouchData {
            self.0.touch.clone().unwrap_or_default()
        }
        pub fn alt_key(&self) -> bool {
            self.touch().alt_key
        }
        pub fn ctrl_key(&self) -> bool {
            self.touch().ctrl_key
        }
        pub fn meta_key(&self) -> bool {
            self.touch().meta_key
        }
        pub fn shift_key(&self) -> bool {
            self.touch().shift_key
        }
        // touches(), target_touches(), changed_touches() would return TouchList
    }

    // ========== DragEvent ==========
    #[derive(Clone, Debug)]
    pub struct DragEvent(pub(crate) MockEvent);
    impl_event_common!(DragEvent);
    impl_mouse_methods!(DragEvent);

    // ========== AnimationEvent ==========
    #[derive(Clone, Debug)]
    pub struct AnimationEvent(pub(crate) MockEvent);
    impl_event_common!(AnimationEvent);
    impl AnimationEvent {
        fn animation(&self) -> MockAnimationData {
            self.0.animation.clone().unwrap_or_default()
        }
        pub fn animation_name(&self) -> String {
            self.animation().animation_name
        }
        pub fn elapsed_time(&self) -> f32 {
            self.animation().elapsed_time
        }
        pub fn pseudo_element(&self) -> String {
            self.animation().pseudo_element
        }
    }

    // ========== TransitionEvent ==========
    #[derive(Clone, Debug)]
    pub struct TransitionEvent(pub(crate) MockEvent);
    impl_event_common!(TransitionEvent);
    impl TransitionEvent {
        fn transition(&self) -> MockTransitionData {
            self.0.transition.clone().unwrap_or_default()
        }
        pub fn property_name(&self) -> String {
            self.transition().property_name
        }
        pub fn elapsed_time(&self) -> f32 {
            self.transition().elapsed_time
        }
        pub fn pseudo_element(&self) -> String {
            self.transition().pseudo_element
        }
    }

    // ========== SubmitEvent ==========
    #[derive(Clone, Debug)]
    pub struct SubmitEvent(pub(crate) MockEvent);
    impl_event_common!(SubmitEvent);

    // ========== UiEvent ==========
    #[derive(Clone, Debug)]
    pub struct UiEvent(pub(crate) MockEvent);
    impl_event_common!(UiEvent);
    impl UiEvent {
        pub fn detail(&self) -> i32 {
            0
        }
    }

    // ========== CompositionEvent ==========
    #[derive(Clone, Debug)]
    pub struct CompositionEvent(pub(crate) MockEvent);
    impl_event_common!(CompositionEvent);
    impl CompositionEvent {
        pub fn data(&self) -> String {
            String::new()
        }
    }

    // ========== ClipboardEvent ==========
    #[derive(Clone, Debug)]
    pub struct ClipboardEvent(pub(crate) MockEvent);
    impl_event_common!(ClipboardEvent);

    // ========== CustomEvent ==========
    #[derive(Clone, Debug)]
    pub struct CustomEvent(pub(crate) MockEvent);
    impl_event_common!(CustomEvent);

    // ========== ErrorEvent ==========
    #[derive(Clone, Debug)]
    pub struct ErrorEvent(pub(crate) MockEvent);
    impl_event_common!(ErrorEvent);
    impl ErrorEvent {
        pub fn message(&self) -> String {
            String::new()
        }
        pub fn filename(&self) -> String {
            String::new()
        }
        pub fn lineno(&self) -> u32 {
            0
        }
        pub fn colno(&self) -> u32 {
            0
        }
    }

    // ========== HashChangeEvent ==========
    #[derive(Clone, Debug)]
    pub struct HashChangeEvent(pub(crate) MockEvent);
    impl_event_common!(HashChangeEvent);
    impl HashChangeEvent {
        pub fn old_url(&self) -> String {
            String::new()
        }
        pub fn new_url(&self) -> String {
            String::new()
        }
    }

    // ========== MessageEvent ==========
    #[derive(Clone, Debug)]
    pub struct MessageEvent(pub(crate) MockEvent);
    impl_event_common!(MessageEvent);
    impl MessageEvent {
        pub fn origin(&self) -> String {
            String::new()
        }
        pub fn last_event_id(&self) -> String {
            String::new()
        }
    }

    // ========== PageTransitionEvent ==========
    #[derive(Clone, Debug)]
    pub struct PageTransitionEvent(pub(crate) MockEvent);
    impl_event_common!(PageTransitionEvent);
    impl PageTransitionEvent {
        pub fn persisted(&self) -> bool {
            false
        }
    }

    // ========== PopStateEvent ==========
    #[derive(Clone, Debug)]
    pub struct PopStateEvent(pub(crate) MockEvent);
    impl_event_common!(PopStateEvent);

    // ========== ProgressEvent ==========
    #[derive(Clone, Debug)]
    pub struct ProgressEvent(pub(crate) MockEvent);
    impl_event_common!(ProgressEvent);
    impl ProgressEvent {
        pub fn length_computable(&self) -> bool {
            false
        }
        pub fn loaded(&self) -> f64 {
            0.0
        }
        pub fn total(&self) -> f64 {
            0.0
        }
    }

    // ========== StorageEvent ==========
    #[derive(Clone, Debug)]
    pub struct StorageEvent(pub(crate) MockEvent);
    impl_event_common!(StorageEvent);
    impl StorageEvent {
        pub fn key(&self) -> Option<String> {
            None
        }
        pub fn old_value(&self) -> Option<String> {
            None
        }
        pub fn new_value(&self) -> Option<String> {
            None
        }
        pub fn url(&self) -> String {
            String::new()
        }
    }

    // ========== BeforeUnloadEvent ==========
    #[derive(Clone, Debug)]
    pub struct BeforeUnloadEvent(pub(crate) MockEvent);
    impl_event_common!(BeforeUnloadEvent);

    // ========== DeviceMotionEvent ==========
    #[derive(Clone, Debug)]
    pub struct DeviceMotionEvent(pub(crate) MockEvent);
    impl_event_common!(DeviceMotionEvent);
    impl DeviceMotionEvent {
        pub fn interval(&self) -> f64 {
            0.0
        }
    }

    // ========== DeviceOrientationEvent ==========
    #[derive(Clone, Debug)]
    pub struct DeviceOrientationEvent(pub(crate) MockEvent);
    impl_event_common!(DeviceOrientationEvent);
    impl DeviceOrientationEvent {
        pub fn alpha(&self) -> Option<f64> {
            None
        }
        pub fn beta(&self) -> Option<f64> {
            None
        }
        pub fn gamma(&self) -> Option<f64> {
            None
        }
        pub fn absolute(&self) -> bool {
            false
        }
    }

    // ========== GamepadEvent ==========
    #[derive(Clone, Debug)]
    pub struct GamepadEvent(pub(crate) MockEvent);
    impl_event_common!(GamepadEvent);

    // ========== PromiseRejectionEvent ==========
    #[derive(Clone, Debug)]
    pub struct PromiseRejectionEvent(pub(crate) MockEvent);
    impl_event_common!(PromiseRejectionEvent);

    // ========== SecurityPolicyViolationEvent ==========
    #[derive(Clone, Debug)]
    pub struct SecurityPolicyViolationEvent(pub(crate) MockEvent);
    impl_event_common!(SecurityPolicyViolationEvent);
    impl SecurityPolicyViolationEvent {
        pub fn document_uri(&self) -> String {
            String::new()
        }
        pub fn referrer(&self) -> String {
            String::new()
        }
        pub fn blocked_uri(&self) -> String {
            String::new()
        }
        pub fn violated_directive(&self) -> String {
            String::new()
        }
        pub fn effective_directive(&self) -> String {
            String::new()
        }
        pub fn original_policy(&self) -> String {
            String::new()
        }
        pub fn disposition(&self) -> String {
            String::new()
        }
        pub fn status_code(&self) -> u16 {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{node_eq, reset_document, testing, MockDom};

    #[test]
    fn html_debugging_works() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        MockDom::set_attribute(&p, "id", "foo");
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, text.as_ref(), None);
        assert_eq!(
            main.to_debug_html(),
            "<main><p id=\"foo\">Hello, world!</p></main>"
        );
    }

    #[test]
    fn remove_attribute_works() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        MockDom::set_attribute(&p, "id", "foo");
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, text.as_ref(), None);
        MockDom::remove_attribute(&p, "id");
        assert_eq!(main.to_debug_html(), "<main><p>Hello, world!</p></main>");
    }

    #[test]
    fn remove_node_works() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        MockDom::set_attribute(&p, "id", "foo");
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, text.as_ref(), None);
        MockDom::remove_node(&main, p.as_ref());
        assert_eq!(main.to_debug_html(), "<main></main>");
    }

    #[test]
    fn insert_before_works() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        let span = MockDom::create_element("span", None);
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&span, text.as_ref(), None);
        MockDom::insert_node(&main, span.as_ref(), Some(p.as_ref()));
        assert_eq!(
            main.to_debug_html(),
            "<main><span>Hello, world!</span><p></p></main>"
        );
    }

    #[test]
    fn insert_before_sets_parent() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        MockDom::insert_node(&main, p.as_ref(), None);
        let parent =
            MockDom::get_parent(p.as_ref()).expect("p should have parent set");
        assert!(node_eq(parent, main));
    }

    #[test]
    fn insert_before_moves_node() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        let span = MockDom::create_element("span", None);
        let text = MockDom::create_text_node("Hello, world!");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&span, text.as_ref(), None);
        MockDom::insert_node(&main, span.as_ref(), Some(p.as_ref()));
        MockDom::insert_node(&main, p.as_ref(), Some(span.as_ref()));
        assert_eq!(
            main.to_debug_html(),
            "<main><p></p><span>Hello, world!</span></main>"
        );
    }

    #[test]
    fn first_child_gets_first_child() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        let span = MockDom::create_element("span", None);
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&p, span.as_ref(), None);
        assert_eq!(
            MockDom::first_child(main.as_ref()).as_ref(),
            Some(p.as_ref())
        );
        assert_eq!(
            MockDom::first_child(&MockDom::first_child(main.as_ref()).unwrap())
                .as_ref(),
            Some(span.as_ref())
        );
    }

    #[test]
    fn next_sibling_gets_next_sibling() {
        let main = MockDom::create_element("main", None);
        let p = MockDom::create_element("p", None);
        let span = MockDom::create_element("span", None);
        let text = MockDom::create_text_node("foo");
        MockDom::insert_node(&main, p.as_ref(), None);
        MockDom::insert_node(&main, span.as_ref(), None);
        MockDom::insert_node(&main, text.as_ref(), None);
        assert_eq!(
            MockDom::next_sibling(p.as_ref()).as_ref(),
            Some(span.as_ref())
        );
        assert_eq!(
            MockDom::next_sibling(span.as_ref()).as_ref(),
            Some(text.as_ref())
        );
    }

    // ========================================================================
    // CSS Selector Matching Tests
    // ========================================================================

    #[test]
    fn test_class_selector_matching() {
        reset_document();
        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "foo");
        MockDom::add_class(&class_list, "bar");

        assert!(testing::matches_selector(&el, ".foo"));
        assert!(testing::matches_selector(&el, ".bar"));
        assert!(testing::matches_selector(&el, ".foo.bar"));
        assert!(testing::matches_selector(&el, ".bar.foo"));
        assert!(!testing::matches_selector(&el, ".baz"));
        assert!(!testing::matches_selector(&el, ".foo.baz"));
    }

    #[test]
    fn test_element_selector_matching() {
        reset_document();
        let button = MockDom::create_element("button", None);
        let div = MockDom::create_element("div", None);

        assert!(testing::matches_selector(&button, "button"));
        assert!(testing::matches_selector(&div, "div"));
        assert!(!testing::matches_selector(&button, "div"));
        assert!(!testing::matches_selector(&div, "button"));
    }

    #[test]
    fn test_id_selector_matching() {
        reset_document();
        let el = MockDom::create_element("div", None);
        MockDom::set_attribute(&el, "id", "my-element");

        assert!(testing::matches_selector(&el, "#my-element"));
        assert!(!testing::matches_selector(&el, "#other"));
    }

    #[test]
    fn test_attribute_selector_matching() {
        reset_document();
        let el = MockDom::create_element("input", None);
        MockDom::set_attribute(&el, "type", "text");
        MockDom::set_attribute(&el, "disabled", "");
        MockDom::set_attribute(&el, "data-value", "hello-world");

        // Existence check
        assert!(testing::matches_selector(&el, "[type]"));
        assert!(testing::matches_selector(&el, "[disabled]"));
        assert!(!testing::matches_selector(&el, "[readonly]"));

        // Exact match
        assert!(testing::matches_selector(&el, "[type=\"text\"]"));
        assert!(!testing::matches_selector(&el, "[type=\"password\"]"));

        // Starts with
        assert!(testing::matches_selector(&el, "[data-value^=\"hello\"]"));
        assert!(!testing::matches_selector(&el, "[data-value^=\"world\"]"));

        // Ends with
        assert!(testing::matches_selector(&el, "[data-value$=\"world\"]"));
        assert!(!testing::matches_selector(&el, "[data-value$=\"hello\"]"));

        // Contains
        assert!(testing::matches_selector(&el, "[data-value*=\"lo-wo\"]"));
        assert!(!testing::matches_selector(&el, "[data-value*=\"xyz\"]"));
    }

    #[test]
    fn test_combined_selector_matching() {
        reset_document();
        let el = MockDom::create_element("button", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "primary");
        MockDom::set_attribute(&el, "type", "submit");
        MockDom::set_attribute(&el, "id", "submit-btn");

        assert!(testing::matches_selector(&el, "button.primary"));
        assert!(testing::matches_selector(&el, "button#submit-btn"));
        assert!(testing::matches_selector(&el, "button[type=\"submit\"]"));
        assert!(testing::matches_selector(&el, ".primary[type=\"submit\"]"));
        assert!(testing::matches_selector(
            &el,
            "button.primary[type=\"submit\"]"
        ));
        assert!(testing::matches_selector(&el, "#submit-btn.primary"));

        assert!(!testing::matches_selector(&el, "div.primary"));
        assert!(!testing::matches_selector(&el, "button.secondary"));
    }

    #[test]
    fn test_descendant_combinator() {
        reset_document();
        let container = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&container);
        MockDom::add_class(&class_list, "container");

        let child = MockDom::create_element("span", None);
        let child_class_list = MockDom::class_list(&child);
        MockDom::add_class(&child_class_list, "text");

        MockDom::insert_node(&container, child.as_ref(), None);

        // Descendant selector should match
        assert!(testing::matches_selector(&child, ".container .text"));
        assert!(testing::matches_selector(&child, "div .text"));
        assert!(testing::matches_selector(&child, ".container span"));

        // Container doesn't match descendant selector
        assert!(!testing::matches_selector(&container, ".container .text"));
    }

    #[test]
    fn test_child_combinator() {
        reset_document();
        let parent = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&parent);
        MockDom::add_class(&class_list, "parent");

        let child = MockDom::create_element("span", None);
        let child_class_list = MockDom::class_list(&child);
        MockDom::add_class(&child_class_list, "child");

        MockDom::insert_node(&parent, child.as_ref(), None);

        // Direct child selector
        assert!(testing::matches_selector(&child, ".parent > .child"));
        assert!(testing::matches_selector(&child, "div > span"));
    }

    #[test]
    fn test_comma_separated_selectors() {
        reset_document();
        let div = MockDom::create_element("div", None);
        let span = MockDom::create_element("span", None);

        // Comma = OR
        assert!(testing::matches_selector(&div, "div, span"));
        assert!(testing::matches_selector(&span, "div, span"));
        assert!(testing::matches_selector(&div, ".foo, div"));
        assert!(!testing::matches_selector(&div, ".foo, .bar"));
    }

    // ========================================================================
    // CSS Cascade Tests
    // ========================================================================

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_basic_cascade() {
        reset_document();
        testing::clear_stylesheets();
        // lightningcss minifies "blue" to "#00f"
        testing::load_stylesheet(".blue { color: blue; } .red { color: red; }");

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "blue");

        // lightningcss minifies color values
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("#00f".to_string())
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_cascade_source_order() {
        reset_document();
        testing::clear_stylesheets();
        testing::load_stylesheet(
            ".item { color: blue; } .item { color: red; }",
        );

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "item");

        // Later rule wins
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("red".to_string())
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_cascade_specificity() {
        reset_document();
        testing::clear_stylesheets();
        testing::load_stylesheet("#high { color: red; } .low { color: blue; }");

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "low");
        MockDom::set_attribute(&el, "id", "high");

        // ID selector has higher specificity
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("red".to_string())
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_cascade_important() {
        reset_document();
        testing::clear_stylesheets();
        testing::load_stylesheet(
            "#high { color: blue; } .low { color: red !important; }",
        );

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "low");
        MockDom::set_attribute(&el, "id", "high");

        // !important overrides higher specificity
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("red".to_string())
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_cascade_inline_style() {
        reset_document();
        testing::clear_stylesheets();
        testing::load_stylesheet(".item { color: blue; }");

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "item");

        let style = MockDom::style(&el);
        MockDom::set_css_property(&style, "color", "green");

        // Inline style wins
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("green".to_string())
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_get_all_computed_styles() {
        reset_document();
        testing::clear_stylesheets();
        // lightningcss minifies color values
        testing::load_stylesheet(
            ".button { color: white; background-color: blue; padding: 10px; }",
        );

        let el = MockDom::create_element("button", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "button");

        let styles = testing::get_all_computed_styles(&el);

        // lightningcss minifies: white -> #fff, blue -> #00f
        assert_eq!(styles.get("color"), Some(&"#fff".to_string()));
        assert_eq!(styles.get("background-color"), Some(&"#00f".to_string()));
        assert_eq!(styles.get("padding"), Some(&"10px".to_string()));
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_tailwind_style_cascade() {
        reset_document();
        testing::clear_stylesheets();

        // Simulate compiled Tailwind output
        // lightningcss minifies rgb values to hex
        testing::load_stylesheet(
            r#"
            .bg-blue-500 { background-color: rgb(59, 130, 246); }
            .text-white { color: rgb(255, 255, 255); }
            .px-4 { padding-left: 1rem; padding-right: 1rem; }
            .py-2 { padding-top: 0.5rem; padding-bottom: 0.5rem; }
            .rounded { border-radius: 0.25rem; }
        "#,
        );

        let button = MockDom::create_element("button", None);
        let class_list = MockDom::class_list(&button);
        MockDom::add_class(&class_list, "bg-blue-500");
        MockDom::add_class(&class_list, "text-white");
        MockDom::add_class(&class_list, "px-4");
        MockDom::add_class(&class_list, "py-2");
        MockDom::add_class(&class_list, "rounded");

        let styles = testing::get_all_computed_styles(&button);

        // lightningcss minifies rgb() to hex
        assert_eq!(
            styles.get("background-color"),
            Some(&"#3b82f6".to_string())
        );
        assert_eq!(styles.get("color"), Some(&"#fff".to_string()));
        assert_eq!(styles.get("padding-left"), Some(&"1rem".to_string()));
        assert_eq!(styles.get("padding-right"), Some(&"1rem".to_string()));
        assert_eq!(styles.get("padding-top"), Some(&".5rem".to_string()));
        assert_eq!(styles.get("border-radius"), Some(&".25rem".to_string()));
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_stylesheet_hash() {
        reset_document();
        testing::clear_stylesheets();

        let hash1 = testing::stylesheet_hash();
        testing::load_stylesheet(".a { color: red; }");
        let hash2 = testing::stylesheet_hash();
        testing::load_stylesheet(".b { color: blue; }");
        let hash3 = testing::stylesheet_hash();

        // Each stylesheet changes the hash
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_multiple_classes_same_property() {
        reset_document();
        testing::clear_stylesheets();

        // Simulate scenario where changing Tailwind config changes color
        // lightningcss minifies rgb() to hex
        testing::load_stylesheet(
            r#"
            .text-primary { color: rgb(59, 130, 246); }
            .text-secondary { color: rgb(100, 116, 139); }
        "#,
        );

        let el = MockDom::create_element("p", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "text-primary");

        // lightningcss minifies rgb(59, 130, 246) to #3b82f6
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("#3b82f6".to_string())
        );

        // Now if Tailwind config changed primary color...
        testing::clear_stylesheets();
        testing::load_stylesheet(
            r#"
            .text-primary { color: rgb(147, 51, 234); }
            .text-secondary { color: rgb(100, 116, 139); }
        "#,
        );

        // The computed style changes! (lightningcss minifies to hex)
        assert_eq!(
            testing::get_computed_style(&el, "color"),
            Some("#9333ea".to_string())
        );
    }

    // ========================================================================
    // Snapshot HTML Tests
    // ========================================================================

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_basic_element() {
        reset_document();
        testing::clear_stylesheets();

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "container");

        let snapshot = el.to_snapshot_html();
        assert_eq!(snapshot, "<div class=\"container\"></div>\n");
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_with_computed_styles() {
        reset_document();
        testing::clear_stylesheets();
        testing::load_stylesheet(".text-blue { color: blue; }");

        let el = MockDom::create_element("span", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "text-blue");

        let snapshot = el.to_snapshot_html();
        // lightningcss minifies blue to #00f
        assert_eq!(
            snapshot,
            "<span class=\"text-blue\" style=\"color: #00f\"></span>\n"
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_nested_elements() {
        reset_document();
        testing::clear_stylesheets();
        testing::load_stylesheet(
            r#"
            .container { padding: 1rem; }
            .btn { background: blue; }
        "#,
        );

        let container = MockDom::create_element("div", None);
        let container_class_list = MockDom::class_list(&container);
        MockDom::add_class(&container_class_list, "container");

        let button = MockDom::create_element("button", None);
        let button_class_list = MockDom::class_list(&button);
        MockDom::add_class(&button_class_list, "btn");
        let text = MockDom::create_text_node("Click me");
        MockDom::insert_node(&button, text.as_ref(), None);
        MockDom::insert_node(&container, button.as_ref(), None);

        let snapshot = container.to_snapshot_html();
        // lightningcss minifies blue to #00f
        assert_eq!(
            snapshot,
            "<div class=\"container\" style=\"padding: 1rem\">\n  <button class=\"btn\" style=\"background: #00f\">\n    Click me\n  </button>\n</div>\n"
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_void_elements() {
        reset_document();
        testing::clear_stylesheets();

        let container = MockDom::create_element("div", None);
        let br = MockDom::create_element("br", None);
        let img = MockDom::create_element("img", None);
        MockDom::set_attribute(&img, "src", "test.png");
        MockDom::set_attribute(&img, "alt", "Test");

        MockDom::insert_node(&container, br.as_ref(), None);
        MockDom::insert_node(&container, img.as_ref(), None);

        let snapshot = container.to_snapshot_html();
        assert_eq!(
            snapshot,
            "<div>\n  <br />\n  <img alt=\"Test\" src=\"test.png\" />\n</div>\n"
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_sorted_attributes() {
        reset_document();
        testing::clear_stylesheets();

        let el = MockDom::create_element("div", None);
        // Add attributes in non-alphabetical order
        MockDom::set_attribute(&el, "data-z", "last");
        MockDom::set_attribute(&el, "data-a", "first");
        MockDom::set_attribute(&el, "id", "test");

        let snapshot = el.to_snapshot_html();
        // Attributes should be sorted alphabetically
        assert_eq!(
            snapshot,
            "<div data-a=\"first\" data-z=\"last\" id=\"test\"></div>\n"
        );
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_detects_css_changes() {
        reset_document();
        testing::clear_stylesheets();

        let el = MockDom::create_element("p", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "text-primary");

        // Initial "Tailwind" CSS
        testing::load_stylesheet(".text-primary { color: rgb(59, 130, 246); }");
        let snapshot1 = el.to_snapshot_html();

        // "Updated Tailwind config" with different primary color
        testing::clear_stylesheets();
        testing::load_stylesheet(".text-primary { color: rgb(147, 51, 234); }");
        let snapshot2 = el.to_snapshot_html();

        // Snapshots should be different
        assert_ne!(snapshot1, snapshot2);
        // lightningcss minifies to hex
        assert!(snapshot1.contains("#3b82f6"));
        assert!(snapshot2.contains("#9333ea"));
    }

    #[cfg(feature = "css_cascade")]
    #[test]
    fn test_snapshot_convenience_function() {
        reset_document();
        testing::clear_stylesheets();

        let el = MockDom::create_element("div", None);
        let class_list = MockDom::class_list(&el);
        MockDom::add_class(&class_list, "test");

        // testing::snapshot should produce same output as to_snapshot_html
        assert_eq!(testing::snapshot(&el), el.to_snapshot_html());
    }
}
