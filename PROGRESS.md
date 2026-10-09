# Adequate UI port progress

The target is a native port suitable for Kane's internal work and prototypes.
Prioritize usable catalogue coverage, common interactions, simple public APIs and
real screen compositions. Visual checks should catch obvious layout/style defects;
exhaustive browser parity, rare edge cases and production certification are not
required. Gallery/example text is English. Do not pursue translated demonstrations.

The repository contains a broad working first pass. The inventory separately tracks
upstream example coverage; API presence does not imply every source preview has
been ported. No component is certified as having exhaustive upstream parity.

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
- Four standalone consuming applications: settings, dashboard, checkout and avatars.
- CLI PNG capture, pinned upstream source harness, Playwright captures, Rust
  side-by-side/difference/content-region comparison tooling.

## Verification checkpoint

- Formatting, workspace compilation, independent library compilation and rustdoc pass.
- All four consuming examples compile.
- 65 focused tests pass using actual Yakui input/layout cycles and pure algorithms.
- 142 native variant routes × two themes, plus six showcase captures and 62 actual-trigger overlay captures: **352 captures,
  zero failures** in the latest full smoke run.
- The configured upstream suite covers 67 actual previews: **284 comparison pairs**
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
preview separately. 67 of 461 source previews currently have paired captures. Keep remaining previews in the inventory and port useful compositions first;
do not map an unported preview to a generic default rendition. Important incomplete areas include:

- Complete documented slots, sizes, disabled/invalid states and compositions.
- Rich navigation flyouts, richer input-group slots, grouped/multiple autocomplete,
  full forms/field association and advanced selector keyboard contracts.
- Drawer drag physics, overlay motion/shadows/delays, tooltip/hover-card pointer transit.
- Calendar caption dropdowns/locales/presets/week numbers; complete data table cells,
  row actions/reordering; complete chart interpolation/axes/tooltips and preview kinds.
- Image/video presentation APIs and chat reactions.
- Complete useful sidebar variants/layouts and common keyboard interactions.
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

## Action menus and popup placement increment

Ported table-actions through public dropdown triggers and menus. Three row menus
return a row/action tuple; default/open screenshots exist for both themes. Added
public popup/menu alignment and offsets, neutral Nova menu highlighting, flush
action rows, labels/icons/shortcuts/inset slots, end-positioned selection indicators,
hover submenu opening and native vertical overflow scrolling. Keyboard opening
chooses first/last enabled items; pointer opening focuses the popup without a
highlight. Checkbox/radio items stay open by default.

Six additional actual native tests cover pointer/key opening, end alignment,
Escape/typeahead, retained checkbox/radio selection, overflow focus reveal and the
actual gallery row callback. Yakui keyboard events do not bubble to parents; a
cached trigger context owns these commands on the focused button. Popup blur,
saturation, soft shadows, exact hover delays/transit and multilingual RTL remain
outstanding. No component was certified complete.

## Pinned Lucide geometry increment

Replaced approximate icons with 137 exact Lucide 0.474.0 vector assets used by
the pinned Base UI examples and primitives. A development-only generator verifies
the shadcn and Lucide pins, normalizes SVG arcs/curves and emits plain Rust paths.
The runtime paints round-capped native strokes; no SVG parser or JavaScript runtime
is part of the library. ISC and Feather MIT notices and source hashes are retained.

The public Icon catalogue and native gallery typography/icons diagnostic render
every asset. Both-theme SVG/native comparisons live in visual/icons separately
from documented preview coverage. The dense grid exposed Yakui's shared u16 mesh
index overflow; clipped icon bounds keep batches bounded without new paint layers.
A real Yakui paint test checks the full grid's indices and finite geometry.
Single-sample capture edges remain visibly less smooth than browser SVG edges.
All 51 tests pass, and 256 native captures have zero failures. No component was
certified complete.

## Documented dropdown composition increment

Ported eleven genuine dropdown previews using public MenuItem trees and native
triggers: default, basic, submenu, shortcuts, icons, checkboxes, checkboxes-icons,
radio-group, radio-icons, destructive and complex. The complex preview's duplicate
notification entries share application booleans. Child popups size from actual
Yakui font measurements; narrow rows wrap between icon and selection indicator.
Root menus use the anchor width by default and scroll within the available side.

Popup content now paints and receives pointer input above its backdrop. Child
menus share the root outside-dismiss target. Escape closes one child level and
returns to its trigger; ordinary actions close the whole menu. Expanded parent
rows retain their highlighted appearance. Eight additional actual-system tests
bring the total to 59; 296 full-suite native captures and 32 targeted native menu
checks pass. Source comparisons include common arrow paths, nested levels, retained
checks/radios and shared notifications. There are 210 component comparison pairs
covering 41 previews, plus the separate icon diagnostic.

`--keys` scripts actual input on both screenshot renderers and records the path.
The bundled OFL Noto Symbols 2 fallback supplies English shortcut symbols; fonts
and licence hashes are reproducible through tools/fetch_fonts.py. Arabic/Hebrew
assets were removed following Kane's English-only decision. Native keyboard
scroll reveal and large nested popup positions can differ from the source harness;
blur, soft shadows, motion and hover transit remain intentional deferred polish
for the internal/prototype target. Avatar triggers remain the next useful gap.

## Texture-backed avatar increment

Added public Avatar/AvatarImage/AvatarSize/AvatarBadge, overlapping avatar_group
and AvatarGroupCount. Applications own decoding, loading state and textures.
Native meshes crop texture UVs to cover the circle; fallback text appears when
the caller supplies no image. Groups overlap by 8px and expose count/icon items.
Buttons accept public show_children composition, including avatar dropdowns.

Ported nine non-directional avatar previews and dropdown-menu-avatar. Both
renderers use the same pinned first-party portrait files, with hashes and capture
substitutions recorded. The reference source/compositions remain unchanged.
The gallery enables JPEG/PNG decoding; the independent library does not depend
on image decoding or a renderer. A fourth consuming app demonstrates app-owned
textures and changing between an image and initials.

Two common-use tests cover real texture geometry, group placement and an avatar
button's pointer activation. All 61 tests and four doctests pass; four apps compile.
The full gallery has 318 successful native captures and no failures. Both-theme
image/fallback/dropdown comparisons bring the component suite to 240 pairs across
51 source previews. Border blending, grayscale rasterisation and badge cascade
differences remain acceptable polish for the internal/prototype target.

## Joined button-group increment

Added horizontal/vertical ButtonGroup, nested group spacing, joined button/input
surfaces, text prefixes and axis-aware separators. Controls retain their normal
state, targets and callbacks. Eight genuine previews cover toolbar, orientation,
sizes, separators, split actions, search input, dropdown and editable popover.
Popup slots provide compact heading/description composition. The screenshot loop
caught a narrow textarea; filling its available width corrected the common path.

Two actual Yakui tests cover geometry, pointer activation, nested spacing and
editing caller-owned text. All 63 tests and four doctests pass. Both-theme default
and open popup pairs bring coverage to 262 pairs across 59 upstream previews.
Nested input-group and select compositions remain pending. The pinned browser
harness does not apply all upstream joining selectors; native groups implement
the documented joining intent. Split uses Lucide Plus rather than Tabler Plus.
Shadow/motion/cascade and exact mixed-size focus stacking are deferred polish.

## Shared input-group increment

Replaced the padded row wrapper with InputGroup and public editor/addon/text/button
slots. Addons choose inline or block placement independently of declaration order.
The layout measures addons first and gives the native editor the remaining width,
with one child layout pass. The group owns its shared focus ring; embedded buttons
retain their own targets. Prefix/icon clicks focus the editor, disabled editing is
blocked, and multiline input uses native Enter/selection behaviour.

Eight genuine previews cover search, icons, text prefixes/suffixes, shortcuts,
loading indicators, leading/trailing fields (including password), and a comment
composer with a Post action. The separate settings app consumes these APIs for
its editable profile/Clear action. Both-theme default and focus comparisons exposed
and corrected an excessive dark fill and a misplaced footer button. Per-example
font/padding/block-height utilities and additional complex compositions are deferred.

Two actual-system tests cover width budgeting, placement, typing, disabled editing,
independent button activation and multiline block layout. All 65 tests and five
doctests pass; all four consuming apps compile. The configured suite has 284 pairs
across 67 upstream previews. No exhaustive-parity claim is made.
