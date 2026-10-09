# Adequate UI port progress

The objective remains the **complete** native port. The repository now contains a
broad working first pass, not a production-complete port. No component is declared
complete. Public API presence and successful rendering do not establish example,
variant or behavioural parity.

## Pinned specification

- shadcn/ui `6ea090075cd537d3b792c6c1a625e2448b6ede26`
- Primary reference: Base UI / Nova / Neutral / Geist
- Yakui `7c2bd75cd00ee83a1d33b8af360788cb322a3df8`
- 65 navigation entries, 64 MDX pages, 461 documented previews
- Form is listed without a Base MDX file. Some newer previews import Base Rhea;
  chart-demo imports new-york-v4. Per-preview imports/styles are in the inventory.
- See RESEARCH.md for source locations, findings and Yakui ownership constraints.

## Implemented

The old implementation was replaced by a renderer-independent `yakui-adequate-ui`
library and a standalone native Winit/WGPU gallery. All 65 catalogue entries have
initial public APIs and native routes. Current behaviour includes:

- Scoped light/dark semantic themes, native typography and small Lucide-style icons.
- Buttons/content/form controls, native Cosmic text editing and masked password
  editing, OTP filtering/paste, sliders and single/multiple toggles.
- Caller-owned overlay state, root placement/collision handling, modal focus trapping,
  autofocus, Escape/backdrop dismissal and return focus.
- Roving tabs/radios, native menu keyboard/typeahead/check/radio/submenu selection,
  searchable command/select/combobox, right-click context anchors.
- Gregorian date arithmetic and calendar single/multiple/range selection/navigation.
- Native table filtering/sorting/visibility/selection/pagination with stable row IDs.
- Native chart meshes/data inspection, carousel drag/arrows/autoplay, resizable
  panels and collapsible sidebar navigation.
- Chat parts, attachment action/state presentation, questionnaires, toast manager,
  transcript auto-follow/prepend preservation and jump-to-message.
- Dashboard, settings and checkout showcases built with public library components.
- Three standalone consuming applications: settings, dashboard and checkout.
- CLI PNG capture, pinned upstream source harness, Playwright captures, Rust
  side-by-side/difference/content-region comparison tooling.

## Verification checkpoint

- Formatting, workspace compilation, independent library compilation and rustdoc pass.
- All three consuming examples compile.
- 44 focused tests pass using actual Yakui input/layout cycles and pure algorithms.
- 107 native variant routes × two themes, plus six showcase captures and 32 actual-trigger overlay captures: **252 captures,
  zero failures** in the latest full smoke run.
- The configured upstream suite covers 29 actual previews: **144 comparison pairs**
  across light/dark and applicable states. Review remains pending; screenshots were
  used to find and correct concrete layout/painting defects, not to certify parity.
- Persistent pairs live under `visual/`; smoke output lives in ignored `visual/local`.
  `reference/verification.json` retains the checkpoint summary and reproduction commands.

The screenshot loop found solid border silhouettes behind transparent controls,
state-hook siblings consuming List spacing, overlaid radio rows, stretched progress
fills/thumbs, and Align expanding natural-width buttons. These were corrected;
focused layout tests cover the radio/progress/button regressions.

## Required work still outstanding

`reference/components.json` records component-specific deviations and every upstream
preview separately. Only 29 of 461 previews currently have paired captures. The
remaining previews must be studied, implemented and compared, not mapped to a
generic default rendition. Important incomplete areas include:

- Complete documented slots, sizes, disabled/invalid states and compositions.
- Rich navigation flyouts, joined button/input groups, grouped/multiple autocomplete,
  full forms/field association and advanced selector keyboard contracts.
- Drawer drag physics, overlay motion/shadows/delays, tooltip/hover-card pointer transit.
- Calendar caption dropdowns/locales/presets/week numbers; complete data table cells,
  row actions/reordering; complete chart interpolation/axes/tooltips and preview kinds.
- Image/video/texture APIs, full icon fidelity, avatar groups/status and chat reactions.
- Complete sidebar variants, layouts, native accessibility audit and bidi/RTL audit.
- Full screenshot/interaction coverage for every component and each upstream preview.
- Final consuming-application, performance and normal-usage audit.

Current notable visual differences include typography/rasterisation, card slot
alignment, textarea inset/resize affordance, and several complex component layouts.
The native capture suite can pass while these differences remain. Do not advertise
this checkpoint as a finished shadcn port.

## Continue development

1. Read this document, RESEARCH.md and `reference/components.json`.
2. Clone both upstreams at `reference/upstream.json` pins. Regenerate the inventory
   with `python3 tools/inventory.py /path/to/shadcn-ui`; recorded evidence is preserved.
3. `cargo run --bin gallery -- --list` lists native routes. Upstream preview names are
   accepted only when the corresponding variant actually exists.
4. `cargo test --workspace`, `cargo check --workspace --examples`, `cargo fmt --all -- --check`.
5. Follow README.md to prepare and start the pinned reference harness.
6. `python3 tools/visual_catalogue.py` reruns configured upstream/native pairs.
7. `python3 tools/verify_gallery.py --all-variants --showcases --open-overlays` reruns native smoke captures.
8. Port the actual next upstream preview; add route, behaviour checks and two-theme
   comparisons, inspect and correct them, then update its explicit coverage record.

Do not run Clippy or add mocked integration tests. Mobile visual validation belongs
to Kane. The project brief authorizes autonomous desktop implementation/verification.

## Checkbox field increment

Ported six actual checkbox previews: demo, basic, description, disabled, invalid
and group. `Checkbox` now exposes a bare indicator, label, helper text, disabled
and invalid styling, font weight and the selectable FieldLabel card composition.
The complete field activates as one native target; wrapping and RTL place the ring
on the indicator. Field legend/description typography is public. Three actual
Yakui tests cover label/helper activation, one Tab stop, Space release, disabled
skipping, bare sizing and mirrored indicator placement.

Thirty-two persistent pairs cover both themes and default/focus/selected states
where supported. Source computed styles establish 14px label leading, 19.25px card
title leading, 21px description leading and the effective 20px group gap. Same
Geist files and exact face matching were confirmed; glyph edge/weight differences
remain visible. Table and multilingual RTL previews remain outstanding.

The comparison runner now preserves other route/state captures when a component
has more than one configured source route. No component was certified complete.

## Table slot increment

Added public `Table`, `TableCell`, header/body/footer/row/head/cell slots, cell
spans, alignment and caption composition. Both `table` and `data_table` now use
this same implementation. Native rows own hover tint; selection remains in caller
state. A clipped horizontal viewport scrolls with wheel/trackpad and reveals newly
focused controls. Column widths are explicit logical values, not an HTML table
layout implementation.

Ported table-demo, table-footer and checkbox-table. The latter uses the exact four
upstream records and initial Sarah Chen selection; its header selects/clears all
rows, and row checkboxes update independently. Three additional tests exercise
actual gallery selection, column/span alignment and horizontal focus reveal. Ten
additional permanent pairs cover both themes, plus checkbox focus/selected states.
Table action menus, multilingual RTL and draggable scrollbar presentation remain
outstanding. Text/border rendering differences remain under visual review.

## Switch field increment

Ported all six non-RTL switch previews: demo, description, choice card, disabled,
invalid and sizes. `Switch` exposes the documented 32×18.4 and 24×14 tracks,
label/helper composition, trailing placement, selectable cards and disabled/invalid
styles. Track and field share one activation/focus target. Field layout and hollow
indicator rings are shared with checkboxes; compact labels now centre vertically.

Two additional real Yakui tests cover helper activation, Space release, one Tab
stop, disabled skipping and exact track sizes. Thirty-two switch pairs replace
the previous four default pairs; checkbox comparisons were rerun after the shared
layout change. Multilingual RTL previews, motion, expanded touch targets and native
platform accessibility remain outstanding. No component is certified complete.
