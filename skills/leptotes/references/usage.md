# leptotes usage

- `render<F, V>(view_fn: F) -> String` where `F: FnOnce() -> V, V:
  RenderHtml`: the view's SSR HTML, built inside a fresh `Owner`. Build
  the view inside the closure; one built before the call creates its
  signals and context outside the owner.
- `count(haystack: &str, needle: &str) -> usize`: non-overlapping
  occurrences of `needle`, for counts such as `count(&html, "<li")`.
- `dom<F, V>(view_fn: F) -> Dom`: `render`, then `Dom::parse`.
- `Dom::parse(html: &str) -> Dom`: parse an HTML fragment. A `Dom` prints
  as that HTML.
- `Dom` and `Element` share five queries by CSS selector, in document
  order; an `Element`'s see only its descendants:
  - `count(sel) -> usize`
  - `find(sel) -> Element`: panics unless exactly one element matches,
    naming the selector and the number found and printing the markup as
    parsed.
  - `find_all(sel) -> Vec<Element>`
  - `texts(sel) -> Vec<String>`: each match's `text()`.
  - `attrs(sel, name) -> Vec<&str>`: each match's value of `name`; a
    match without it is left out.
- `Element::text_content() -> String`: its `textContent`, whitespace
  kept (adjacent elements join with nothing between), the `<!>` markers
  left out. For a `<pre>` or a `<style>`. It is what a browser's parser
  yields: a newline directly after `<pre>` or `<textarea>` is dropped, CR
  and CRLF become LF, and entities are decoded except in an HTML `<style>`
  or `<script>` (one inside an `<svg>` is decoded).
- `Element::text() -> String`: `text_content()` with whitespace trimmed
  and collapsed. A non-breaking space stays.
- `:scope > *` selects the top-level elements of a `Dom`, or the children
  of an `Element`. `dom.find(":scope > *")` is the component's one root
  element, or a panic if it renders more than one. A whole document's top
  level is what its `<head>` and `<body>` held.
- `Element::attr(name) -> Option<&str>`. An `Element`'s queries do not
  see the element itself. It prints as its own markup, serialized again,
  so a `<!>` marker prints as `<!---->`.
- Every query panics on a selector scraper does not implement, naming
  it. That includes state pseudo-classes (`:checked`, `:disabled`,
  `:hover`) and pseudo-elements; select on the attribute (`[checked]`).
- The fragment is parsed as a `<template>`'s contents: a bare `<tr>` or
  `<td>` keeps its tags, and a `<table>` gets a `<tbody>`, so select
  `table tr`, not `table > tr`.
- A fragment whose first element only SVG has (`<g>`, `<line>`,
  `<marker>`) is parsed as an `<svg>`'s contents, keeping `<image>` and
  camelCase attributes. One that starts with `<a>`, `<script>`, `<style>`,
  or `<title>` is parsed as HTML; wrap it in an `<svg>`.
- A whole document loses its `<html>`, `<head>`, and `<body>` tags.
- Queries and the text methods skip what is inside a `<template>` element.
