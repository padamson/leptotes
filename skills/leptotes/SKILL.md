---
name: leptotes
description: Use when writing or reviewing native unit tests for Leptos components with the leptotes crate (`leptotes::render`, `leptotes::count`), when a component test should run under plain `cargo test` or nextest without a browser, or when deciding whether a check belongs in a leptotes test or a playwright-rust end-to-end test.
license: MIT OR Apache-2.0
metadata:
  version: "0.1.0"
  internal: true
---

# leptotes

<!--
Hidden from installers (`metadata.internal: true`) until the API settles;
nothing loads it yet. Bump `metadata.version` on every content change.
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
