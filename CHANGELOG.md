# Changelog

All notable changes to this project are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added
- `render`: build a Leptos view inside a fresh reactive owner and return its SSR HTML string, hydration markers included
- `count`: the number of non-overlapping occurrences of a substring, for counting rendered elements
- `dom`, `Dom`, and `Element`: render a view and query its HTML by CSS selector (`count`, `find`, `find_all`, `texts`, `attrs`), reading an element's text with the hydration markers left out
- `Element::text_content`: an element's text with its whitespace kept, for a `<pre>` or a `<style>`

[Unreleased]: https://github.com/padamson/leptotes/commits/main
