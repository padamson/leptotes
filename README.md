# leptotes

Native component tests for [Leptos](https://leptos.dev): render a
component to its HTML in a plain `#[test]` and assert on the result, with
no browser and no `wasm-bindgen-test` runner.

**Status:** early. The API is two functions, `render` and `count`, and it
grows only when a component test needs something more. It is not on
crates.io yet.

## What it does

`leptotes::render` builds a view inside a fresh reactive owner and returns
its server-side (SSR) HTML string, the way Leptos renders in production,
so signals and context created in the view work as they do there. The test
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
times</button>`). Compare exactly only where none appears, and count or
match a fragment elsewhere.

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
