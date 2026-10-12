# yakui-adequate-ui

A native Rust port of shadcn/ui built directly on Yakui, targeting English internal
interfaces and prototypes. The agreed scope is a usable full catalogue with common
examples and documented gaps; exhaustive reproduction of all upstream demos is
outside that target. The prototype common-use audit is complete. See PROGRESS.md
and reference/components.json for verified coverage and limitations.

The design reference is pinned shadcn Base UI / Nova / Neutral. The library depends
on no windowing or rendering backend. The gallery uses official Yakui Winit/WGPU
and `yakui-to-image` integrations.

```sh
cargo run --bin gallery
cargo run --bin gallery -- --list
cargo run --bin gallery -- --component button --variant outline --theme dark \
  --isolated --width 600 --height 400 --screenshot /tmp/button.png
cargo run --bin gallery -- --component dialog --theme dark --state open \
  --isolated --screenshot /tmp/dialog.png
cargo run --bin gallery -- --showcase dashboard --theme dark
cargo run -p adequate-gallery --example settings
cargo test --workspace
cargo fmt --all -- --check
cargo check -p yakui-adequate-ui
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

Read COMPONENTS.md for API selection, THEMING.md for overrides, ANIMATIONS.md for
motion and deterministic sequence capture, and AGENTS.md for
agent usage and development rules. `ui::add_fonts` loads bundled Geist under OFL;
font and Lucide licence notices are retained. Shadcn's MIT notice is preserved in
reference/SHADCN-LICENSE. Source pins are in reference/upstream.json.

Avatar captures use the same image assets from the pinned shadcn checkout on both
renderers. They replace mutable GitHub portrait URLs only in the capture harness;
source widgets and compositions stay unchanged. Captures record the substitutions,
and `crates/gallery/assets/avatars/provenance.json` records hashes/licences.
`--component avatar --variant basic --state image-error --screenshot /tmp/fallback.png`
shows the real initials fallback. The `avatars` consuming example owns JPEG decoding
and texture creation; the library accepts Yakui texture IDs.
