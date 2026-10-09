# Adequate UI port progress

The objective remains the **complete** native port. This document records evidence,
not a reduced definition of done. No component is currently declared complete.

## Pinned specification

- shadcn/ui `6ea090075cd537d3b792c6c1a625e2448b6ede26`
- Base UI, Nova style, Neutral tokens, Geist font family
- Yakui `7c2bd75cd00ee83a1d33b8af360788cb322a3df8`
- `reference/components.json`: 65 navigation entries, 461 documented preview references
- Upstream lists `form` in Base navigation, but has no matching documentation file.
  Preserve this entry until its intended scope is resolved from upstream history.

## Implemented foundation

- Replaced the legacy crates; the public crate is `yakui-adequate-ui`.
- Renderer-independent library, direct Yakui layout/paint/input.
- Scoped light/dark themes, semantic tokens, OKLCH conversion, bundled OFL Geist.
- Native Winit/WGPU gallery with catalogue navigation and window resizing.
- Headless native PNG capture, with real hover/focus input and explicit viewport.
- Pinned upstream source harness (Vite/Base UI), Playwright capture.
- Rust side-by-side/difference generator with JSON measurements.
- Real Yakui event-cycle tests for pointer, Enter/Space, cancellation, disabled,
  focus traversal, caller-owned checkbox/text state, Unicode editing, tokens/sizes.

## Work in progress

Basic component APIs/examples, keyboard groups, and overlay focus/dismissal.
Their existence is **not** full parity: examples and variants are tracked separately
in the inventory. Current card example still differs from the pinned login example.

## Remaining required work

- All 461 upstream previews mapped to native, independent gallery examples.
- Complete variant/state APIs and focused behavioural checks per component.
- Complete menus, selects, comboboxes, OTP and advanced form controls.
- Data tables, calendar/date picker, charts, carousel, sidebar/resizable layouts.
- New message/attachment/bubble/marker/questionnaire/scroller catalogue.
- Light/dark capture pairs, review and correction for the entire catalogue.
- Dashboard, settings and form-heavy showcase compositions.
- Rust API documentation and separate consuming examples.
- Final integration/visual audit: **do not publish as production-complete yet**.

## Continue development

1. Read this document, `reference/upstream.json` and `reference/components.json`.
2. Clone the upstream repositories at the recorded revisions; regenerate inventory
   with `python3 tools/inventory.py /path/to/shadcn-ui` (preserves coverage fields).
3. `cargo run --bin gallery -- --list` reports implemented capture routes.
4. `cargo test --workspace` and `cargo fmt --all -- --check`.
5. Follow the capture/comparison instructions in README.md.
6. Port the actual upstream example; update coverage only after tests and inspection.

Do not run Clippy. Do not add mocked integration tests. Preserve Kane's architecture
ownership; the project brief explicitly authorizes autonomous implementation here.
