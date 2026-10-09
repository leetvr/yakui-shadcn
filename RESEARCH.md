# Pinned references and native mechanisms

shadcn/ui is pinned at `6ea090075cd537d3b792c6c1a625e2448b6ede26`.
Yakui is pinned at `7c2bd75cd00ee83a1d33b8af360788cb322a3df8`.
The repository URLs, source roots and pins are machine-readable in
`reference/upstream.json`. MIT notices and bundled font licences are retained.

## What defines the port

The Base navigation in `apps/v4/content/docs/components/base/meta.json` contains
65 entries. The 64 available MDX pages reference 461 distinct previews. `form`
is listed without a Base MDX file; it remains in the inventory. The component
list and preview list are extracted from the pinned checkout, rather than from
memory. `tools/inventory.py` rejects a checkout at any other revision.

The primary styling reference is `apps/v4/registry/styles/style-nova.css`, paired
with `apps/v4/registry/bases/base/ui` and Neutral tokens from
`apps/v4/registry/themes.ts`. Examples live in `apps/v4/examples/base`.
Some newer chat examples import Base Rhea; chart-demo imports new-york-v4.
Each preview's imports and style are recorded individually. A Nova rendition of
these mixed examples still needs an explicit comparison and deviation review.

Source studies include Button/Card/Field/Input/Checkbox/Radio/Switch, overlay and
menu composition, Calendar/Date Picker, Data Table, Chart/Carousel/Sidebar,
Attachment/Bubble/Message/Marker/Questionnaire and Message Scroller. These studies
establish initial mechanisms; they do not certify full example parity.

The full Next website also builds the CLI/registry monorepo. For repeatable captures,
`tools/prepare_reference.py` copies unmodified pinned widgets and examples into a
small Vite harness, loads the actual Nova stylesheet and tokens, and uses the same
Geist files as the native library. It expands IconPlaceholder to the selected
Lucide icon as the upstream registry build does. The harness records explicit
viewport, theme, interaction state and bounds. Desktop form comparisons use a
1000px viewport so upstream's `md:text-sm` matches native desktop typography.

## Yakui ownership and ordering

The library emits ordinary Yakui nodes and paint commands. Its dependencies do
not include Winit, WGPU or Vulkan. The gallery owns the window and renderer.
Application values own selection, text, dates, table settings, transcripts and
popup visibility. Retained widgets own pointer gestures, focus appearance and
unconsumed events. A response transfers a pending edit back to application state
on the next frame.

Yakui widget identity includes call site, type and tree position. Declare a
component at the same source call site each frame. Moving a declaration between
branches can reset retained interaction state. Test frames use the same builders
for this reason.

During event dispatch Yakui mutably borrows the DOM node arena. Reading a Scope
from inside an event can borrow the arena again and panic. Custom widgets capture
scoped contexts during update, then use the captured handles during events.

Focus navigation is applied at frame completion. FocusChanged is delivered on
the following update. Root overlay scopes select an initial control, constrain
modal Tab traversal, handle dismissal, and return focus to their recorded trigger.
Popup content is a root sibling of scrolling content so clipping cannot hide it.

A headless Yakui instance needs both surface size and an explicit unscaled
viewport. Setting only surface size yields a 1px capture. Normal window integration
sets the viewport through Yakui/Winit. Captures use logical scale 1.

A Scope or generic container overlays children by default. A standalone `use_state`
node also consumes List spacing; retained state now owns each component container. Use an explicit row or
column for multiple sequential children. Tight constraints also propagate through
RoundRect: align an inner thumb or progress fill before giving it a smaller size.
These mistakes were found with real rendered comparisons and corrected.

Yakui's RoundedRectangle border first paints a filled silhouette. A clear inset
exposes the silhouette across the entire control. Transparent bordered surfaces
therefore resolve their interior against the containing surface; focus rings use
a hollow outline mesh. Semantic alpha tokens are resolved in sRGB to reproduce CSS
compositing across linear game renderers. Raw backdrop alpha and image/text edges
still require backend-specific visual audit.

## Verification is evidence, not certification

Real Yakui event/layout tests exercise editing, activation, cancellation, focus,
selection, date arithmetic, scrolling anchors and navigation. There are no mocked
integration tests. Native smoke captures detect renderer/layout failures. Upstream
pairs add side-by-side images, difference images and both viewport/content error
measurements. All pairs require human or agent visual review; a small metric or a
successful capture is not a completed component.

Password controls keep plaintext in application/retained editing state and paint only
bullets. Grapheme indices own selection and deletion. A clipped masked viewport
measures bullet advances for pointer/caret/scroll coordinates. Paste uses Yakui
clipboard facilities; copy/cut do not expose password text.

## Checkbox field source checks

Studied all eight pinned `examples/base/checkbox-*.tsx` files, Base checkbox/field
primitives and Nova CSS. Computed styles from the source harness matter: Label and
FieldLabel resolve to 14px leading (Label's rule overrides the nominal snug class);
FieldTitle resolves to 19.25px. Group label font-normal and gap-3 are overridden in
the compiled cascade by effective weight 500 and gap 20px. Match that rendered
reference rather than inferring geometry from class strings alone.

Both browser and native use the identical Geist files. Cosmic shaping was checked
to select Geist-Medium at weight 500 and the same 187.698px label advance as the
browser's 187.703125px. Remaining perceived weight differences arise after shaping
and require a rendering audit. Do not substitute heavier font weights to mask them.

## Native table slots

Pinned TableHeader is 40px high; ordinary rows contain 8px cell padding, 20px text
and a bottom border. TableBody omits the last border; TableFooter adds a top border
and muted/50 fill. Browser collapsed borders split fractional geometry, so native
rows differ by up to half a pixel. The rendered Nova heading text-left overrides
the demo Amount heading's text-right class; numeric body cells still align right.
Gallery columns were measured at an 800px table width. Native applications specify
column widths directly and retain arbitrary component content in cells.

Yakui's pinned Scrollable supports only vertical scrolling. The table viewport
therefore clips a full-width native canvas, clamps horizontal offset, bubbles wheel
input at boundaries and reveals focused descendants. Before measuring a new focus
rectangle, its previous frame's scroll translation must be reset to zero.

## Switch source checks

Studied pinned Base switch primitive, Nova switch/thumb tokens and every switch
example. Default track is 32×18.4 with a 16px thumb; small is 24×14 with a 12px thumb.
Unchecked dark input uses the semantic input alpha multiplied by 0.8. Description
and invalid examples put content before the trailing switch; choice cards use
FieldTitle's 19.25px leading and the checked FieldLabel surface. Invalid indicator
rings use destructive/20 in light and destructive/40 in dark.

Field layout keeps the indicator at the first helper line; bare compact labels
centre against the taller track. Card focus decorates the card, while ordinary
field focus decorates only the track. Both patterns use one native activation and
focus target and write the application boolean from the consumed response.

## Dropdown trigger and action table mechanisms

Pinned Base UI moves focus to the menu popup after a pointer opening, without
highlighting a row. ArrowDown/Up enters first/last enabled items. Checkbox/radio
items default to closeOnClick=false, whereas ordinary items close. Native menus
now preserve these distinctions and restore the recorded trigger on dismissal.
The table action preview uses three independent MenuState values and returns row
and action IDs to gallery state, verified through actual key/pointer cycles.

Yakui dispatches keys only to the focused widget; EventResponse::Bubble allows
global navigation, not parent key dispatch. Dropdown trigger commands therefore
live in a scoped Rc<Cell> captured by ControlWidget during update. Reading scopes
inside events remains forbidden by the DOM borrow mechanism. Menu roots themselves
own pointer-opening keyboard input; enabled entries own subsequent navigation.

Nova's final cn-menu-translucent rule overrides destructive text to the accent
foreground and highlighted rows to foreground/10; separators become foreground/5.
These measured cascade results are reproduced. Popup blur/saturation and soft
shadow are still absent: the pinned renderer-independent Yakui paint interface
has no framebuffer backdrop-filter operation. Menus currently use an opaque
semantic popup surface. Placement exposes side/align/offset, logical RTL alignment
and all four side flips/clamping. Native vertical scrolling reveals focused rows.

## Pinned icon assets

The pinned apps/v4/package.json uses lucide-react 0.474.0. Every Lucide import and
IconPlaceholder in first-party Base previews and primitives is extracted from that
package. Source nodes, hashes and source paths are preserved in
crates/adequate-ui/icons/provenance.json; original ISC/Feather MIT notices are in
icons/LICENSE. Regenerate with node tools/reference-app/generate-icons.mjs followed
by the path to the pinned shadcn checkout. The development-only SVG normalization
uses [svgpath 2.6.0](https://github.com/fontello/svgpath). Native runtime code contains
only sampled paths and Yakui meshes, with round stroke caps and joins.

The diagnostic compares all 137 SVGs against native geometry in both themes at
24 logical pixels. It is not an additional documented shadcn preview. The native
single-sample renderer produces visibly different curve edges; all paths and
alignment match. Dense geometry needs separate clipped bounds because pinned
Yakui PaintDom merges consecutive compatible meshes without checking u16 overflow.
