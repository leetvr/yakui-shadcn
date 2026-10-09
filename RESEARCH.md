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
