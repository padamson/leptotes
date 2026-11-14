//! Minimal component testing for Leptos.
//!
//! `leptotes` (Greek δοκιμή, "a test / proof") renders a Leptos view to an HTML
//! string under a fresh reactive owner so components can be asserted on in fast,
//! native unit tests — no browser, no `wasm-bindgen-test` runner. It is the
//! cheap inner-loop complement to the playwright-rust end-to-end gate.
//!
//! Incubated in the slp workspace and dogfooded against `slp-ui` / `theoria`;
//! kept framework-only (no slp deps) so it can graduate to its own crate. Its
//! capabilities are **pulled by demand** — a new helper lands when an slp (or
//! theoria) component test needs it. See `docs/PLAN.md`.
//!
//! ```
//! use leptos::prelude::*;
//!
//! let html = leptotes::render(|| view! { <p>"hello"</p> });
//! assert_eq!(html, "<p>hello</p>");
//! ```

use leptos::prelude::*;
use leptos::tachys::view::RenderHtml;

/// Render a Leptos view to its server-side HTML string.
///
/// Takes a closure so the view (and any signals it creates) is constructed
/// inside the temporary [`Owner`], matching how Leptos renders in production.
///
/// The string is Leptos's SSR output as is, hydration markers included: reactive
/// text next to static text comes out as `<button>Clicked <!>3<!> times</button>`.
/// An exact comparison holds only where no marker appears.
#[must_use]
pub fn render<F, V>(view_fn: F) -> String
where
    F: FnOnce() -> V,
    V: RenderHtml,
{
    let owner = Owner::new();
    owner.with(|| view_fn().to_html())
}

/// Number of non-overlapping occurrences of `needle` in `haystack`. Handy for
/// counting rendered elements, e.g. `count(&html, "<line")`.
#[must_use]
pub fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[cfg(test)]
mod tests {
    //! leptotes testing leptotes — the tool proves itself.
    use super::*;

    #[test]
    fn renders_static_markup_to_html() {
        assert_eq!(render(|| view! { <p>"hello"</p> }), "<p>hello</p>");
    }

    #[test]
    fn renders_the_current_value_of_a_signal() {
        let html = render(|| {
            let n = RwSignal::new(3);
            view! { <span>{move || n.get()}</span> }
        });
        assert_eq!(html, "<span>3</span>");
    }

    #[test]
    fn reactive_text_beside_static_text_keeps_its_hydration_markers() {
        let html = render(|| {
            let n = RwSignal::new(3);
            view! { <button>"Clicked " {move || n.get()} " times"</button> }
        });
        assert_eq!(html, "<button>Clicked <!>3<!> times</button>");
    }

    #[test]
    fn the_view_sees_context_provided_inside_render() {
        // Without the owner, `provide_context` has nowhere to store the
        // value and `use_context` falls back to -1.
        let html = render(|| {
            provide_context(42_i32);
            view! { <span>{use_context::<i32>().unwrap_or(-1)}</span> }
        });
        assert_eq!(html, "<span>42</span>");
    }

    #[test]
    fn count_does_not_count_overlapping_matches() {
        assert_eq!(count("aaaa", "aa"), 2);
    }
}
