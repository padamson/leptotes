# leptotes

Native component tests for [Leptos](https://leptos.dev): render a
component to its HTML in a plain `#[test]` and assert on the result, with
no browser and no `wasm-bindgen-test` runner.

**Status:** early. The API is `render`, `count`, and `dom` with its
queries, and it grows only when a component test needs something more. It
is not on crates.io yet.

## What it does

`leptotes::render` builds a view inside a fresh reactive owner and returns
its server-side (SSR) HTML string, the way Leptos renders in production,
so signals and context created in the view work as they do there.
`leptotes::dom` renders the same way and parses the HTML (with
[scraper](https://crates.io/crates/scraper)), so a test can ask for
elements by CSS selector and read their text and attributes. The test
runner is whatever you already use (`cargo test` or `cargo nextest run`).

It sees the initial render only. Clicks, layout, visibility, and anything
that happens after hydration need a real browser, so they belong in an
end-to-end test (for example with
[playwright-rust](https://github.com/padamson/playwright-rust)).
[`docs/PLAN.md`](docs/PLAN.md) draws that line in more detail and lists
what may come next.

## Installation

Until the first release, take it from git as a dev-dependency:

```toml
[dev-dependencies]
leptotes = { git = "https://github.com/padamson/leptotes" }
```

The component crate needs Leptos's `ssr` feature in its test build.

## Usage

```rust
use leptos::prelude::*;

#[test]
fn greeting_renders_the_name() {
    let html = leptotes::render(|| view! { <p>"hello"</p> });
    assert_eq!(html, "<p>hello</p>");
}

#[test]
fn list_renders_one_item_per_entry() {
    let html = leptotes::render(|| {
        view! { <ul>{["a", "b", "c"].into_iter().map(|i| view! { <li>{i}</li> }).collect_view()}</ul> }
    });
    assert_eq!(leptotes::count(&html, "<li>"), 3);
}
```

The string is Leptos's SSR output as is, so it keeps the `<!>` hydration
markers Leptos writes beside reactive content (`<button>Clicked <!>3<!>
times</button>`). Compare it exactly only where none appears.

### Querying by selector

`leptotes::dom` returns a `Dom`. Its queries take a CSS selector and see
elements in document order:

| Query | Returns |
|---|---|
| `count(sel)` | how many elements match (0 when none do) |
| `find(sel)` | the one `Element` that matches; panics otherwise, printing the HTML as parsed |
| `find_all(sel)` | every `Element` that matches |
| `texts(sel)` | the text of each match |
| `attrs(sel, name)` | the value of `name` on each match that has it |

An `Element` has `text()` and `attr(name)`, and the same five queries,
which see only its descendants. `text()` is the element's `textContent`
with the `<!>` markers gone, entities decoded, and whitespace trimmed and
collapsed (a non-breaking space stays), so it compares exactly. Like
`textContent`, it joins adjacent elements with nothing between
(`<h1>A</h1><p>B</p>` reads `AB`). A `Dom`
prints as the rendered HTML, so `"{dom}"` makes a failure message show what
was rendered.

```rust
use leptos::prelude::*;

#[test]
fn the_menu_marks_the_build_being_viewed() {
    let dom = leptotes::dom(|| view! {
        <select>
            <option value="dev">"dev"</option>
            <option value="0.14.0" selected>"v0.14.0"</option>
        </select>
    });
    assert_eq!(dom.attrs("option", "value"), ["dev", "0.14.0"], "{dom}");
    assert_eq!(dom.find("option[selected]").text(), "v0.14.0", "{dom}");
}
```

What the parser does with the HTML:

- It is parsed the way a browser parses a `<template>`'s contents, so a
  component whose root is a `<tr>` or `<td>` keeps its tags.
- A fragment whose first element is one only SVG has (`<g>`, `<line>`,
  `<marker>`) is parsed as the contents of an `<svg>`, so `<image>` and
  attributes such as `refX` come through as written. A fragment that
  starts with `<a>`, `<script>`, `<style>`, or `<title>`, which HTML has
  too, is parsed as HTML; wrap it in an `<svg>` to query it as SVG.
- A `<table>` gets the `<tbody>` a browser adds, so `table > tr` matches
  nothing and `table tr` matches the row.
- A whole document loses its `<html>`, `<head>`, and `<body>` tags (and
  their attributes); what they held is still there to query.
- Queries and `text()` skip what is inside a `<template>` element, as a
  browser's do.

Selectors are the ones scraper implements. It has no state pseudo-classes
(`:checked`, `:disabled`, `:hover`) and no pseudo-elements, and a query
with one panics; select on the attribute instead (`[checked]`,
`[disabled]`).

## Development

See [CLAUDE.md](CLAUDE.md) for development commands.

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (MSRV: 1.88)
- [prek](https://github.com/j178/prek) for pre-commit hooks: `cargo install prek && prek install --overwrite`

### Build and test

```bash
cargo build
cargo nextest run
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
