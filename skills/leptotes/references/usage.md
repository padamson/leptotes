# leptotes usage

- `render<F, V>(view_fn: F) -> String` where `F: FnOnce() -> V, V:
  RenderHtml`: the view's SSR HTML, built inside a fresh `Owner`. Build
  the view inside the closure; one built before the call creates its
  signals and context outside the owner.
- `count(haystack: &str, needle: &str) -> usize`: non-overlapping
  occurrences of `needle`, for counts such as `count(&html, "<li")`.
