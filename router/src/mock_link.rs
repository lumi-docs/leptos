//! Mock version of the link module for mock_dom feature.
//! Provides the ToHref trait without browser dependencies.

/// Describes a value that is either a static or a reactive URL, i.e.,
/// a [`String`], a [`&str`], or a reactive `Fn() -> String`.
pub trait ToHref {
    /// Converts the (static or reactive) URL into a function that can be called to
    /// return the URL.
    fn to_href(&self) -> Box<dyn Fn() -> String + '_>;
}

impl ToHref for &str {
    fn to_href(&self) -> Box<dyn Fn() -> String> {
        let s = self.to_string();
        Box::new(move || s.clone())
    }
}

impl ToHref for String {
    fn to_href(&self) -> Box<dyn Fn() -> String> {
        let s = self.clone();
        Box::new(move || s.clone())
    }
}

impl ToHref for std::borrow::Cow<'_, str> {
    fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
        let s = self.to_string();
        Box::new(move || s.clone())
    }
}

impl ToHref for leptos::oco::Oco<'_, str> {
    fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
        let s = self.to_string();
        Box::new(move || s.clone())
    }
}

impl ToHref for std::rc::Rc<str> {
    fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
        let s = self.to_string();
        Box::new(move || s.clone())
    }
}

impl<F> ToHref for F
where
    F: Fn() -> String + 'static,
{
    fn to_href(&self) -> Box<dyn Fn() -> String + '_> {
        Box::new(self)
    }
}

use leptos::children::Children;
use leptos::oco::Oco;
use leptos::prelude::*;
use reactive_graph::computed::ArcMemo;

/// Mock A component for testing - renders as a simple <a> element.
///
/// This is a simplified version of the real A component that doesn't
/// require browser APIs. It renders an anchor tag with the href but
/// doesn't do client-side navigation or active link detection.
#[component]
pub fn A<H>(
    /// Used to calculate the link's `href` attribute.
    href: H,
    /// Where to display the linked URL.
    #[prop(optional, into)]
    target: Option<Oco<'static, str>>,
    /// If `true`, the link is marked active when the location matches exactly.
    #[prop(optional)]
    exact: bool,
    /// If `true`, and when `href` has a trailing slash, `aria-current` will only be set
    /// if `current_url` also has a trailing slash.
    #[prop(optional)]
    strict_trailing_slash: bool,
    /// If `true`, the router will scroll to the top of the window at the end of navigation.
    #[prop(default = true)]
    scroll: bool,
    /// The nodes or elements to be shown inside the link.
    children: Children,
) -> impl IntoView
where
    H: ToHref + Send + Sync + 'static,
{
    // Suppress unused variable warnings for props we don't use in mock mode
    let _ = (exact, strict_trailing_slash, scroll);

    // Get the initial href string - in mock mode we don't need reactive path resolution
    let href_string = href.to_href()();
    let href_memo = ArcMemo::new(move |_| href_string.clone());

    view! {
        <a href=move || href_memo.get() target=target>
            {children()}
        </a>
    }
}
