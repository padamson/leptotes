# D4 — Query the SSR output by CSS selector

*Pulled by: pyle's version bar, hero, and landing-page tests, and theoria's
controls and story-nav tests, which each parse the HTML by hand (option
values in order, the text of a link, a subtree by class) or count substrings
that also match unrelated text (`count(&html, "active")`).*

## Story

As a Leptos developer, I want to ask for elements by CSS selector and read
their text and attributes, so that a component test states what it checks
and fails with the markup in front of me.

## Vertical slices

- **D4.0 — `Dom` and its queries**
  - [x] `dom(|| view! { … })` renders as `render` does and parses the HTML
  - [x] `count`, `find` (exactly one match or a panic that prints the HTML),
        and `find_all`, in document order
  - [x] `texts` and `attrs` for lists of values
  - [x] `Element::text` without hydration markers, entities decoded,
        whitespace collapsed; `Element::attr`
  - [x] an `Element`'s queries see only its descendants

## Notes / refs

- Parsed with scraper (html5ever and Servo's `selectors`), as a
  `<template>`'s contents rather than scraper's default `<body>`, so a bare
  `<tr>` or `<td>` keeps its tags; as an `<svg>`'s contents when the first
  element is one only SVG has, since slp renders SVG parts on their own;
  and with scripting off, so a `<noscript>`'s children are elements.
- The SVG context is a guess from the first tag's whole name, against a
  list of elements only SVG has. A fragment that starts with `<a>`,
  `<script>`, `<style>`, or `<title>`, or puts a bare SVG part after an
  HTML root, is parsed as HTML. If the guess fails a consumer again, an
  explicit `Dom::parse_svg` replaces it.
- Queries and `text()` skip a `<template>` element's contents, which
  scraper's traversals reach and a browser's do not.
- A whole document is not handled as one: its `<html>`, `<head>`, and
  `<body>` tags are dropped. A document parse is the fix when a test of an
  app shell needs them.
- The shape follows what bUnit, Testing Library, and Vitest share: a strict
  single-element query whose failure shows the markup, a scoped query, and
  normalized text. `texts` and `attrs` follow Floki's `attribute/3`.
- `count(&html, needle)` stays for substring counts outside HTML, such as a
  stylesheet.
- Next on this path: D3 (assertions with readable failures) is smaller now
  that `find` panics with the markup; D7 (role queries) and a semantic
  markup comparison (bUnit's `MarkupMatches`) are candidates when a test
  needs them. D5 (re-render after a signal changes) needs a new handle that
  keeps the reactive owner alive, since `render` and `dom` drop it on
  return.
