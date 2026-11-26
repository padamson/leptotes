//! Rendered HTML, parsed so a test can query it by CSS selector.

use std::fmt;
use std::ops::Deref;

use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{ParseOpts, QualName, driver, local_name, ns};
use leptos::tachys::view::RenderHtml;
use scraper::{ElementRef, Html, HtmlTreeSink, Selector};

/// Render a view as [`render`](crate::render) does and parse the HTML for
/// querying.
///
/// ```
/// use leptos::prelude::*;
///
/// let dom = leptotes::dom(|| {
///     let n = RwSignal::new(3);
///     view! { <button class="primary">"Clicked " {move || n.get()} " times"</button> }
/// });
/// assert_eq!(dom.find("button.primary").text(), "Clicked 3 times", "{dom}");
/// ```
#[must_use]
pub fn dom<F, V>(view_fn: F) -> Dom
where
    F: FnOnce() -> V,
    V: RenderHtml,
{
    Dom::from_html(crate::render(view_fn))
}

/// Rendered HTML, parsed so a test can query it by CSS selector.
///
/// Every query takes a CSS selector and sees the elements in document
/// order, leaving out what is inside a `<template>`, as a browser's queries
/// do. A `Dom` prints as the HTML it was parsed from, `<!>` markers and all,
/// so `"{dom}"` in an assertion's message shows what was rendered.
pub struct Dom {
    html: String,
    tree: Html,
}

impl Dom {
    /// Parse an HTML fragment, such as the string [`render`](crate::render)
    /// returns.
    ///
    /// A fragment whose first element is one only SVG has (`<g>`, `<line>`,
    /// `<marker>`) is parsed as the contents of an `<svg>`, and any other as
    /// the contents of a `<template>`. Either way the `<html>`, `<head>`, and
    /// `<body>` tags of a whole document are dropped.
    #[must_use]
    pub fn parse(html: &str) -> Self {
        Self::from_html(html.to_owned())
    }

    fn from_html(html: String) -> Self {
        // A `<template>` rather than scraper's `<body>`, so that a component
        // whose root is a `<tr>` or `<td>` keeps its tags, as it does in a
        // browser's `template.innerHTML`.
        let context = if first_tag(&html).is_some_and(is_svg_only) {
            QualName::new(None, ns!(svg), local_name!("svg"))
        } else {
            QualName::new(None, ns!(html), local_name!("template"))
        };
        // Scripting off, or a `<noscript>`'s children are text.
        let options = ParseOpts {
            tree_builder: TreeBuilderOpts {
                scripting_enabled: false,
                ..TreeBuilderOpts::default()
            },
            ..ParseOpts::default()
        };
        let sink = HtmlTreeSink::new(Html::new_fragment());
        let tree = driver::parse_fragment(sink, options, context, Vec::new(), false).one(&*html);
        Self { html, tree }
    }

    fn root(&self) -> Element<'_> {
        Element(self.tree.root_element())
    }

    /// How many elements match `selector`.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn count(&self, selector: &str) -> usize {
        self.root().count(selector)
    }

    /// The one element that matches `selector`.
    ///
    /// # Panics
    ///
    /// Unless exactly one element matches, naming the selector and the
    /// number found and printing the HTML as it was parsed; or if `selector`
    /// is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn find(&self, selector: &str) -> Element<'_> {
        one(self.root(), selector, || self.root().0.inner_html())
    }

    /// Every element that matches `selector`, none included.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn find_all(&self, selector: &str) -> Vec<Element<'_>> {
        self.root().find_all(selector)
    }

    /// The [text](Element::text) of each element that matches `selector`.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn texts(&self, selector: &str) -> Vec<String> {
        self.root().texts(selector)
    }

    /// The value of attribute `name` on each element that matches
    /// `selector`. A match without the attribute is left out.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn attrs(&self, selector: &str, name: &str) -> Vec<&str> {
        self.root().attrs(selector, name)
    }
}

impl fmt::Display for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.html)
    }
}

impl fmt::Debug for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Dom").field(&self.html).finish()
    }
}

/// One element of a [`Dom`]. Its queries see only its descendants, not the
/// element itself. It prints as its own markup, serialized again from the
/// parsed tree, so a `<!>` marker prints as `<!---->`.
#[derive(Clone, Copy)]
pub struct Element<'a>(ElementRef<'a>);

impl<'a> Element<'a> {
    /// The element's text content, as the DOM's `textContent` reads it: the
    /// text of every descendant joined with nothing between, so
    /// `<h1>A</h1><p>B</p>` reads `AB`. Entities are decoded, comments
    /// (Leptos's `<!>` markers among them) are left out, and HTML whitespace
    /// is trimmed and collapsed to single spaces. A non-breaking space is
    /// not HTML whitespace, so it stays.
    #[must_use]
    pub fn text(&self) -> String {
        let raw: String = self
            .0
            .descendants()
            .filter(|node| !in_template(node))
            .filter_map(|node| node.value().as_text())
            .map(|text| &**text)
            .collect();
        raw.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// The value of attribute `name`, or `None` if the element lacks it.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&'a str> {
        self.0.attr(name)
    }

    /// How many of the element's descendants match `selector`.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn count(&self, selector: &str) -> usize {
        self.find_all(selector).len()
    }

    /// The one descendant that matches `selector`.
    ///
    /// # Panics
    ///
    /// Unless exactly one descendant matches, naming the selector and the
    /// number found and printing this element's markup; or if `selector` is
    /// not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn find(&self, selector: &str) -> Element<'a> {
        one(*self, selector, || self.to_string())
    }

    /// Every descendant that matches `selector`, none included.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn find_all(&self, selector: &str) -> Vec<Element<'a>> {
        self.0
            .select(&parse_selector(selector))
            .filter(|found| !in_template(found))
            .map(Element)
            .collect()
    }

    /// The [text](Element::text) of each descendant that matches `selector`.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn texts(&self, selector: &str) -> Vec<String> {
        self.find_all(selector).iter().map(Element::text).collect()
    }

    /// The value of attribute `name` on each descendant that matches
    /// `selector`. A match without the attribute is left out.
    ///
    /// # Panics
    ///
    /// If `selector` is not a selector leptotes supports.
    #[must_use]
    #[track_caller]
    pub fn attrs(&self, selector: &str, name: &str) -> Vec<&'a str> {
        self.find_all(selector)
            .iter()
            .filter_map(|element| element.attr(name))
            .collect()
    }
}

impl fmt::Display for Element<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.html())
    }
}

impl fmt::Debug for Element<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Element").field(&self.0.html()).finish()
    }
}

// ego-tree's `NodeRef`, which scraper does not re-export.
type Node<'a> = <ElementRef<'a> as Deref>::Target;

// scraper keeps a `<template>`'s contents under a fragment node below the
// element, where its traversals reach them and a browser's do not.
fn in_template(node: &Node<'_>) -> bool {
    node.ancestors()
        .any(|ancestor| ancestor.value().is_fragment() && ancestor.parent().is_some())
}

/// The name of the first element in `html`.
fn first_tag(html: &str) -> Option<&str> {
    html.split('<')
        .skip(1)
        .find(|rest| rest.starts_with(|c: char| c.is_ascii_alphabetic()))
        .map(|rest| {
            // The whole name, so that `line-chart` is not `line`.
            let end = rest
                .find(|c: char| c.is_ascii_whitespace() || c == '/' || c == '>')
                .unwrap_or(rest.len());
            &rest[..end]
        })
}

/// Whether SVG has an element of this name and HTML does not. `a`, `script`,
/// `style`, and `title` are in both, so a fragment that starts with one is
/// parsed as HTML.
fn is_svg_only(name: &str) -> bool {
    const SVG_ONLY: [&str; 33] = [
        "animate",
        "animateMotion",
        "animateTransform",
        "circle",
        "clipPath",
        "defs",
        "desc",
        "ellipse",
        "filter",
        "foreignObject",
        "g",
        "image",
        "line",
        "linearGradient",
        "marker",
        "mask",
        "metadata",
        "mpath",
        "path",
        "pattern",
        "polygon",
        "polyline",
        "radialGradient",
        "rect",
        "set",
        "stop",
        "switch",
        "symbol",
        "text",
        "textPath",
        "tspan",
        "use",
        "view",
    ];
    // The filter primitives: `feBlend`, `feGaussianBlur`, and the rest.
    let filter_primitive = name
        .strip_prefix("fe")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_uppercase()));
    filter_primitive || SVG_ONLY.contains(&name)
}

#[track_caller]
fn parse_selector(selector: &str) -> Selector {
    match Selector::parse(selector) {
        Ok(parsed) => parsed,
        Err(error) => {
            panic!("leptotes: `{selector}` is not a selector leptotes supports: {error:?}")
        }
    }
}

#[track_caller]
fn one<'a>(scope: Element<'a>, selector: &str, markup: impl FnOnce() -> String) -> Element<'a> {
    match scope.find_all(selector).as_slice() {
        [only] => *only,
        found => panic!(
            "leptotes: expected exactly one element matching `{selector}`, found {}, in:\n{}",
            found.len(),
            markup()
        ),
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{Dom, Element, dom};

    fn list() -> Dom {
        Dom::parse(r#"<ul><li id="a">one</li><li class="x">two</li><li id="c">three</li></ul>"#)
    }

    // The `p` and `b` outside the `div` are what an `Element` query that
    // looked at the whole document would find. They come first so that the
    // document's HTML does not begin with the `div`'s markup, which would
    // let a scoped panic printing the whole document pass for one printing
    // the element.
    fn nested() -> Dom {
        Dom::parse(
            r#"<p id="out" class="x">outside</p><b>out</b><div><p id="in">inside</p><b>bold</b></div>"#,
        )
    }

    #[test]
    fn count_is_the_number_of_elements_a_selector_matches() {
        assert_eq!(list().count("li"), 3);
    }

    #[test]
    fn count_is_zero_when_nothing_matches() {
        assert_eq!(list().count("ol"), 0);
    }

    #[test]
    fn find_returns_the_one_element_a_selector_matches() {
        assert_eq!(list().find("li.x").text(), "two");
    }

    #[test]
    #[should_panic(expected = "matching `li.y`, found 0")]
    fn find_panics_when_no_element_matches() {
        let _ = list().find("li.y");
    }

    #[test]
    #[should_panic(expected = "matching `li`, found 3")]
    fn find_panics_when_several_elements_match() {
        let _ = list().find("li");
    }

    // The parser's `<tbody>` is why `table > tr` matches nothing, and the
    // HTML as written would not show it.
    #[test]
    #[should_panic(expected = "<table><tbody><tr><td>cell</td></tr></tbody></table>")]
    fn find_s_panic_prints_the_html_as_parsed() {
        let _ = Dom::parse("<table><tr><td>cell</td></tr></table>").find("table > tr");
    }

    thread_local! {
        static PANICKED_AT: std::cell::RefCell<Option<(String, u32)>> =
            const { std::cell::RefCell::new(None) };
        static RECORDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    // `#[track_caller]` on every function between the query and its
    // `panic!` is what puts the test's own line in the report. Nothing else
    // checks it, and a panic moved into a closure would lose it.
    //
    // The hook is the whole process's, and under `cargo test` other tests
    // panic on other threads while it is in place, so it records only for
    // this thread and hands every other panic to the hook it replaced.
    #[test]
    fn a_failed_find_reports_the_line_that_called_it() {
        let previous = std::sync::Arc::new(std::panic::take_hook());
        let theirs = std::sync::Arc::clone(&previous);
        std::panic::set_hook(Box::new(move |info| {
            if RECORDING.get() {
                let at = info.location().map(|l| (l.file().to_owned(), l.line()));
                PANICKED_AT.with(|cell| *cell.borrow_mut() = at);
            } else {
                theirs(info);
            }
        }));
        let dom = list();
        RECORDING.set(true);
        let line = line!() + 1;
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dom.find("li.y")));
        RECORDING.set(false);
        std::panic::set_hook(Box::new(move |info| previous(info)));
        assert_eq!(
            PANICKED_AT.with(|cell| cell.borrow().clone()),
            Some((file!().to_owned(), line))
        );
    }

    #[test]
    #[should_panic(expected = "`li[` is not a selector leptotes supports")]
    fn a_query_with_a_selector_leptotes_cannot_use_panics_naming_it() {
        let _ = list().count("li[");
    }

    #[test]
    fn find_all_returns_the_matches_in_document_order() {
        let dom = list();
        let ids: Vec<_> = dom
            .find_all("li[id]")
            .iter()
            .map(|li| li.attr("id"))
            .collect();
        assert_eq!(ids, [Some("a"), Some("c")]);
    }

    #[test]
    fn texts_is_the_text_of_each_match_in_document_order() {
        assert_eq!(list().texts("li"), ["one", "two", "three"]);
    }

    #[test]
    fn attrs_leaves_out_a_match_without_the_attribute() {
        assert_eq!(list().attrs("li", "id"), ["a", "c"]);
    }

    #[test]
    fn text_leaves_out_the_hydration_markers_beside_reactive_text() {
        let dom = dom(|| {
            let n = RwSignal::new(3);
            view! { <button>"Clicked " {move || n.get()} " times"</button> }
        });
        assert_eq!(dom.find("button").text(), "Clicked 3 times", "{dom}");
    }

    #[test]
    fn text_trims_and_collapses_whitespace() {
        let dom = Dom::parse("<p>\n  two   words \n</p>");
        assert_eq!(dom.find("p").text(), "two words");
    }

    #[test]
    fn text_decodes_entities() {
        let dom = Dom::parse("<span>Signal&lt;f64&gt;</span>");
        assert_eq!(dom.find("span").text(), "Signal<f64>");
    }

    #[test]
    fn text_joins_adjacent_elements_with_nothing_between() {
        let dom = Dom::parse("<div><h1>Title</h1><p>Body</p></div>");
        assert_eq!(dom.find("div").text(), "TitleBody");
    }

    #[test]
    fn text_keeps_a_non_breaking_space() {
        let dom = Dom::parse("<span>10&nbsp;km</span>");
        assert_eq!(dom.find("span").text(), "10\u{a0}km");
    }

    #[test]
    fn text_leaves_out_what_is_inside_a_template() {
        let dom = Dom::parse("<div>shown<template>hidden</template></div>");
        assert_eq!(dom.find("div").text(), "shown");
    }

    #[test]
    fn a_query_does_not_see_inside_a_template() {
        let dom = Dom::parse("<template><p>hidden</p></template><p>shown</p>");
        assert_eq!(dom.texts("p"), ["shown"]);
    }

    #[test]
    fn a_noscript_s_children_are_elements() {
        let dom = Dom::parse(r#"<noscript><p class="warn">Enable JS</p></noscript>"#);
        assert_eq!(dom.texts("noscript p.warn"), ["Enable JS"], "{dom}");
    }

    // Parsed as HTML, `<image>` becomes `<img>`.
    #[test]
    fn a_fragment_that_starts_with_an_svg_element_is_parsed_as_svg() {
        let dom = Dom::parse(r#"<g><image href="a.png"></image></g>"#);
        assert_eq!(dom.attrs("g > image", "href"), ["a.png"], "{dom}");
    }

    // Parsed as HTML, `refX` becomes `refx`.
    #[test]
    fn an_svg_part_keeps_the_case_of_its_attributes() {
        let dom = Dom::parse(r#"<marker refX="5"></marker>"#);
        assert_eq!(dom.attrs("marker", "refX"), ["5"], "{dom}");
    }

    #[test]
    fn a_filter_primitive_is_an_svg_element() {
        let dom = Dom::parse(r#"<feGaussianBlur stdDeviation="2"></feGaussianBlur>"#);
        assert_eq!(dom.attrs("feGaussianBlur", "stdDeviation"), ["2"], "{dom}");
    }

    #[test]
    fn the_first_element_decides_past_a_hydration_marker() {
        let dom = Dom::parse(r#"<!><marker refX="5"></marker>"#);
        assert_eq!(dom.attrs("marker", "refX"), ["5"], "{dom}");
    }

    // Parsed as SVG, a custom element's HTML children are moved out of it.
    #[test]
    fn a_custom_element_named_like_an_svg_element_is_parsed_as_html() {
        let dom = Dom::parse("<line-chart><b>in</b></line-chart>");
        assert_eq!(dom.texts("line-chart > b"), ["in"], "{dom}");
    }

    #[test]
    fn a_custom_element_that_starts_with_fe_is_parsed_as_html() {
        let dom = Dom::parse("<feed-list><b>in</b></feed-list>");
        assert_eq!(dom.texts("feed-list > b"), ["in"], "{dom}");
    }

    // Parsed as SVG, `<input>` is no void element and would hold the text.
    #[test]
    fn a_fragment_that_starts_with_an_html_element_is_parsed_as_html() {
        let dom = Dom::parse("<input>after");
        assert_eq!(dom.find("input").text(), "", "{dom}");
    }

    #[test]
    fn a_whole_document_loses_its_html_head_and_body_tags() {
        let dom = Dom::parse(
            r#"<!DOCTYPE html><html><head><title>T</title></head><body class="b"><p>x</p></body></html>"#,
        );
        assert_eq!(
            (dom.count("html, head, body"), dom.texts("title, p")),
            (0, vec!["T".to_string(), "x".to_string()]),
            "{dom}"
        );
    }

    #[test]
    fn text_includes_the_text_of_descendants() {
        let dom = Dom::parse("<p>a <b>bold</b> word</p>");
        assert_eq!(dom.find("p").text(), "a bold word");
    }

    #[test]
    fn a_table_row_outside_a_table_keeps_its_tags() {
        let dom = Dom::parse("<tr><td>cell</td></tr>");
        assert_eq!(dom.count("tr > td"), 1, "{dom}");
    }

    // The README warns that the parser adds a `<tbody>`.
    #[test]
    fn a_table_row_inside_a_table_keeps_its_tags_under_a_tbody() {
        let dom = Dom::parse("<table><tr><td>cell</td></tr></table>");
        assert_eq!(dom.count("table > tbody > tr > td"), 1, "{dom}");
    }

    #[test]
    fn attr_is_the_attribute_s_value() {
        assert_eq!(list().find("li.x").attr("class"), Some("x"));
    }

    #[test]
    fn attr_is_none_when_the_element_lacks_the_attribute() {
        assert_eq!(list().find("li.x").attr("id"), None);
    }

    #[test]
    fn a_dom_prints_as_the_html_it_was_parsed_from() {
        let html = crate::render(|| {
            let n = RwSignal::new(3);
            view! { <p>"n is " {move || n.get()}</p> }
        });
        assert_eq!(Dom::parse(&html).to_string(), html);
    }

    #[test]
    fn an_element_prints_as_its_own_markup() {
        assert_eq!(
            nested().find("div").to_string(),
            r#"<div><p id="in">inside</p><b>bold</b></div>"#
        );
    }

    #[test]
    fn an_element_prints_a_hydration_marker_as_an_empty_comment() {
        let dom = Dom::parse("<p>a<!>b</p>");
        assert_eq!(dom.find("p").to_string(), "<p>a<!---->b</p>");
    }

    #[test]
    fn a_dom_debugs_as_its_html() {
        assert_eq!(
            format!("{:?}", Dom::parse("<p>a</p>")),
            r#"Dom("<p>a</p>")"#
        );
    }

    #[test]
    fn an_element_debugs_as_its_markup() {
        let dom = Dom::parse("<p>a</p>");
        assert_eq!(format!("{:?}", dom.find("p")), r#"Element("<p>a</p>")"#);
    }

    #[test]
    fn an_element_s_count_sees_only_its_descendants() {
        let dom = nested();
        assert_eq!(dom.find("div").count("p, b"), 2);
    }

    #[test]
    fn an_element_s_queries_do_not_see_the_element_itself() {
        let dom = nested();
        assert_eq!(dom.find("div").count("div"), 0);
    }

    #[test]
    fn an_element_s_find_sees_only_its_descendants() {
        let dom = nested();
        assert_eq!(dom.find("div").find("p").attr("id"), Some("in"));
    }

    #[test]
    #[should_panic(expected = "in:\n<div><p id=\"in\">inside</p><b>bold</b></div>")]
    fn an_element_s_find_panic_prints_the_element_s_markup() {
        let dom = nested();
        let _ = dom.find("div").find("p.x");
    }

    #[test]
    fn an_element_s_find_all_sees_only_its_descendants() {
        let dom = nested();
        let found: Vec<_> = dom
            .find("div")
            .find_all("p")
            .iter()
            .map(Element::text)
            .collect();
        assert_eq!(found, ["inside"]);
    }

    #[test]
    fn an_element_s_texts_sees_only_its_descendants() {
        let dom = nested();
        assert_eq!(dom.find("div").texts("p"), ["inside"]);
    }

    #[test]
    fn an_element_s_attrs_sees_only_its_descendants() {
        let dom = nested();
        assert_eq!(dom.find("div").attrs("p", "id"), ["in"]);
    }
}
