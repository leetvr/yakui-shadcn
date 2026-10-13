# yakui-shadcn

An unofficial Rust port of [shadcn/ui](https://ui.shadcn.com/) to
[Yakui](https://github.com/SecondHalfGames/yakui), targeting English internal
interfaces and prototypes. The component designs, visual language and reference
examples come from shadcn/ui and its contributors, with behavior based on
[Base UI](https://base-ui.com/). This repository contributes the Rust/Yakui
implementation; it is not an original UI design system or an official shadcn project.
Upstream source and its MIT notice are retained in
[reference/upstream.json](reference/upstream.json) and
[reference/SHADCN-LICENSE](reference/SHADCN-LICENSE).

The agreed scope is a usable full catalogue with common
examples and documented gaps; exhaustive reproduction of all upstream demos is
outside that target. The prototype common-use audit is complete. See `reference/components.json` for verified coverage and limitations.

The design reference is pinned shadcn Base UI / Nova / Neutral. The library depends
on no windowing or rendering backend. Games declare and paint UI every vsync
frame; idle-host scheduling and non-vsync UIs are outside scope. The gallery shares its component tree between native Winit/WGPU and browser WebGPU hosts.

## WebGPU demo

[Open the gallery](https://leetvr.github.io/yakui-shadcn/). It renders the same Rust
components into a canvas using WebGPU and a continuous `requestAnimationFrame`
loop, following OATG's web deployment. Use a browser with WebGPU enabled.
Narrow screens use a compact header and component picker. Swipe to scroll the
page, and pan wide previews horizontally; sliders and split handles retain their
drag gestures. Text fields open the software keyboard, and the gallery resizes
to the visible viewport while keeping the focused field in view. Safe-area insets
are respected. Mobile visual validation remains a manual device check.
Each preview has an animated Code panel with highlighted Rust, the state it uses,
its helpers, and Copy. Snippets are extracted from the compiled examples at build
time. Loading, progress, notification stacks and carousel autoplay have motion demos.
The dashboard staggers its cards, counts up metrics and reveals its chart on entry;
Replay animation repeats the sequence while preserving table selection.
Query parameters select examples, for example
`?component=dialog&theme=dark`, `?component=button&variant=outline&isolated`, or
`?showcase=dashboard`. Browser text input supports hardware and software keyboards, including insertion,
deletion and line breaks. Browser autocorrection is disabled because Cosmic owns
the document and selection. Editor clipboard integration, full IME composition,
touch selection handles and inertial scrolling remain outside this demo (code
panels support Copy).

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
wasm-pack build crates/gallery --target web --release --out-dir ../../web/pkg --locked
python3 -m http.server 8080 --directory web
```

Open `http://localhost:8080`. GitHub Actions builds and deploys `web/` to Pages on
pushes to `main`; generated Wasm assets are ignored by Git.

## Using the library

Use `yakui_shadcn as ui` and call `ui::add_fonts(&mut yak)` before the first
frame. Application state owns values and overlay state. Declare components at
stable call sites between `yak.start()` and `yak.finish()` each vsync frame, then
paint with your renderer. All Yakui crates must use the fork revision in `Cargo.toml`.

```rust
use yakui_shadcn as ui;
ui::with_theme(ui::Theme::dark(), || {
    ui::column(12., || {
        ui::checkbox("VSync", &mut settings.vsync);
        if ui::button("Apply").clicked { apply_settings(); }
    });
});
```

Prefer existing components. Use `row`/`column` for sequential children; generic
Yakui containers overlay them. Themes are ordinary values scoped with `with_theme`;
change semantic color, radius, spacing and typography fields on `Theme::light()`
or `Theme::dark()`. Keep root overlays outside scroll areas and open them with the
actual trigger response so dismissal returns focus.

Motion uses a shared frame clock and interruptible tweens. Keep overlay content
alive through exit transitions. Use `animation::animate` for interpolated values,
`presence` for retained exits and `motion` for paint transforms; paint transforms
do not move hit bounds. `with_motion_enabled(false, ...)` settles transitions and
`with_animation_time(Duration, ...)` enables deterministic captures. Spring physics,
drag inertia and non-vsync hosts are outside scope. Source-level API documentation
and `reference/components.json` retain component details and known gaps.

Capture a real motion sequence with:

```sh
python3 tools/motion_capture.py --component switch --state default \
  --keys Tab,Space --theme dark --duration-ms 192
```

## Native demo and verification

```sh
cargo run --bin gallery
cargo run --bin gallery -- --list
cargo run --bin gallery -- --component button --variant outline --theme dark \
  --isolated --width 600 --height 400 --screenshot /tmp/button.png
cargo run --bin gallery -- --component dialog --theme dark --state open \
  --isolated --screenshot /tmp/dialog.png
cargo run --bin gallery -- --showcase dashboard --theme dark
cargo run -p shadcn-gallery --example settings
cargo test --workspace
cargo fmt --all -- --check
cargo check -p yakui-shadcn
```

The live WGPU gallery and PNG captures use 4× MSAA. `--width` and `--height`
are logical dimensions; add `--scale-factor 2` to capture at Retina density.
For matched upstream captures use `tools/visual.py --scale-factor 2`.

Capture states: `default`, `hover`, `focus`, `disabled`, `selected`, `open`.
`open` activates the actual isolated example trigger (hover for tooltips) and checks popup state; use
`--isolated` for reproducible trigger captures. Unknown/unported
routes return errors. A normal gallery shows all 65 catalogue entries with their
current native variants. The current target is internal work and prototypes, with English examples.
The inventory records remaining upstream examples and differences. `--isolated` matches the upstream preview
viewport without gallery navigation. On macOS the screenshot process needs access
to Metal; a sandbox without GPU access cannot capture. Add `--keys ArrowDown,Enter`
to send actual key events after the initial state. For example, `--component
dropdown-menu --variant checkboxes --state open --keys ArrowDown,Enter` opens the
menu and toggles its first checkbox.

## Upstream capture and comparison

```sh
git clone https://github.com/shadcn-ui/ui .upstream/shadcn
git -C .upstream/shadcn checkout 6ea090075cd537d3b792c6c1a625e2448b6ede26
python3 tools/prepare_reference.py .upstream/shadcn
npm ci --prefix tools/reference-app
./tools/reference-app/node_modules/.bin/playwright install chromium
npm run dev --prefix tools/reference-app
```

In another terminal:

```sh
cargo build --workspace
node tools/reference-app/capture.mjs button-outline dark /tmp/upstream.png focus
cargo run --bin gallery -- --component button --variant outline --theme dark \
  --state focus --isolated --width 600 --height 400 --screenshot /tmp/native.png
cargo run --bin compare -- /tmp/upstream.png /tmp/native.png /tmp/comparison
python3 tools/visual.py
```

The local reference harness uses unmodified pinned component/example files, the
upstream Nova stylesheet, shadcn custom CSS variants, and semantic tokens, and the exact same Geist files as
the native library. Its explicit reference routes are configured in `reference/visual-routes.json`;
expand those routes and the harness loader list as each actual example is ported. It is a
reproducible substitute for the upstream Next website's much larger registry/CLI
build, not a handwritten recreation of shadcn styles.

`compare` writes `side-by-side.png`, `content-side-by-side.png`, `difference.png`
and `metrics.json`. Captures
are matched by logical dimensions, pixel density (1× by default), theme and actual interaction state. Metrics
require visual review and do not automatically certify fidelity. `visual/` retains configured paired reference/native captures in both themes.
Run `python3 tools/visual_catalogue.py` for all configured upstream pairs and
`python3 tools/verify_gallery.py --all-variants --showcases --open-overlays` for native smoke captures.
These are separate forms of evidence; neither automatically certifies a component.

Regenerate the native Lucide assets from the installed, pinned package with
`node tools/reference-app/generate-icons.mjs .upstream/shadcn`. The generator
retains source nodes, hashes and licences. Run the separate icon diagnostic with
`python3 tools/visual.py --component typography --variants icons --reference-example
lucide-catalogue --states default --width 1000 --height 800 --output visual/icons`.

See `AGENTS.md` for development rules. `ui::add_fonts` loads bundled Geist under OFL;
font and Lucide licence notices are retained. Shadcn's MIT notice is preserved in
reference/SHADCN-LICENSE. Source pins are in reference/upstream.json.

Avatar captures use the same image assets from the pinned shadcn checkout on both
renderers. They replace mutable GitHub portrait URLs only in the capture harness;
source widgets and compositions stay unchanged. Captures record the substitutions,
and `crates/gallery/assets/avatars/provenance.json` records hashes/licences.
`--component avatar --variant basic --state image-error --screenshot /tmp/fallback.png`
shows the real initials fallback. The `avatars` consuming example owns JPEG decoding
and texture creation; the library accepts Yakui texture IDs.

The gallery bundles Apache-2.0 Roboto Mono for source panels; its license and
provenance are in `crates/gallery/assets/fonts/`.
