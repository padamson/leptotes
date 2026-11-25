---
name: leptotes
description: Use when writing or reviewing native unit tests for Leptos components with the leptotes crate (`leptotes::render`, `leptotes::count`), when writing a new component's first-render tests before the component and its CSS, when a component test should run under plain `cargo test` or nextest without a browser, or when deciding whether a check belongs in a leptotes test or a playwright-rust end-to-end test.
license: MIT OR Apache-2.0
metadata:
  version: "0.2.0"
  internal: true
---

# leptotes

<!--
Hidden from installers (`metadata.internal: true`) until the API settles.
Bump `metadata.version` on every content change.
-->

## When to use

A Leptos component needs a test that runs in milliseconds under `cargo
test` or `cargo nextest run`, with no browser and no `wasm-bindgen-test`
runner.

## How it works

`leptotes::render(|| view! { ... })` builds the view inside a fresh
reactive `Owner` and returns its SSR HTML string; the test asserts on that
string. Signals and context created inside the closure work as they do in
production. The output keeps Leptos's `<!>` hydration markers, so compare
exactly only where none appears.

leptotes sees the initial render only. Clicks, layout, visibility and
anything after hydration belong in a playwright-rust end-to-end test.

- [`references/usage.md`](references/usage.md): the API surface.

## A new component, tests first

Anything the component decides without a view (an ordering, a label,
which parts to show) goes in a plain function with its own unit tests
first. Then, before the component or its CSS exists, write leptotes tests
for its first render:

1. **Its parts, in order.** Each part is present
   (`count(&html, r#"<h1 class="my-hero-title">"#)` is 1), and one comes
   before the next (compare `html.find` positions). Render an empty
   version too, and check that the optional parts are absent.
2. **A site's classes after the component's own.** A `class="tall"` prop
   renders as `class="my-hero tall"`, so a site's rules build on the
   component's.
3. **Every class the markup carries has a rule in the stylesheet.**
   Collect the names inside each `class="..."` in the HTML, and assert
   that the list of those with no `.name {` in the stylesheet is empty.
   One helper serves every component's test.

Then write the component and its CSS until those pass.

## Running the tests

Run them with the features the repo's hook and CI pass, not
`--all-features`. With Leptos's `csr` feature on, an `Effect` the
component creates tries to spawn on an executor `render` never starts,
and the test panics ("tried to spawn a Future with
`Executor::spawn_local()` before a global executor was initialized").
Under `ssr` the effect does not run.
