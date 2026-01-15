#![allow(unused)]

//! A stupidly-simple mock DOM implementation that can be used for testing.
//!
//! Do not use this for anything real. This module provides a simple mock DOM
//! that can be used for unit testing renderer methods without requiring a browser.
//!
//! For component snapshot testing, use SSR (server-side rendering) instead.

use super::CastFrom;
use indexmap::IndexMap;
use slotmap::{new_key_type, SlotMap};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

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

/// Mock event type (unit type for testing).
pub type Event = ();

/// Mock class list (unit type for testing).
#[derive(Clone, Debug, Default)]
pub struct ClassList;

/// Mock CSS style declaration (unit type for testing).
#[derive(Clone, Debug, Default)]
pub struct CssStyleDeclaration;

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
            } => {
                buf.push('<');
                buf.push_str(tag);
                for (k, v) in attrs {
                    buf.push(' ');
                    buf.push_str(k);
                    buf.push_str("=\"");
                    buf.push_str(v);
                    buf.push('"');
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
pub fn reset_document() {
    DOCUMENT.with(|d| d.reset());
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
    pub fn set_attribute(node: &Element, name: &str, value: &str) {
        Document::with_node_mut(node.0 .0, |node| {
            if let NodeType::Element { ref mut attrs, .. } = node.ty {
                attrs.insert(name.to_string(), value.to_string());
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
            if let NodeType::Element { ref mut children, .. } = node.ty {
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
    pub fn create_element_from_html(_html: &str) -> Element {
        document().create_element("div")
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

#[cfg(test)]
mod tests {
    use super::{node_eq, MockDom};

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
}
