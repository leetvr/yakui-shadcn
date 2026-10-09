# yakui-adequate-ui

A native Rust port of shadcn/ui built directly on Yakui. **The full port is in
progress; this is not a production-complete catalogue yet.** See PROGRESS.md and
reference/components.json for verified coverage and outstanding work.

The design reference is pinned shadcn Base UI / Nova / Neutral. The library depends
on no windowing or rendering backend. The gallery uses official Yakui Winit/WGPU
and `yakui-to-image` integrations.

```sh
cargo run --bin gallery
cargo run --bin gallery -- --list
cargo run --bin gallery -- --component button --variant outline --theme dark \
  --isolated --width 600 --height 400 --screenshot /tmp/button.png
cargo run --bin gallery -- --component dialog --theme dark --state selected \
  --isolated --screenshot /tmp/dialog.png
cargo test --workspace
cargo fmt --all -- --check
cargo check -p yakui-adequate-ui
```

Capture states: `default`, `hover`, `focus`, `disabled`, `selected`. Unknown/unported
routes return errors. A normal gallery shows the entire catalogue, explicitly
labelling pending implementations. `--isolated` matches the upstream preview
viewport without gallery navigation. On macOS the screenshot process needs access
to Metal; a sandbox without GPU access cannot capture.

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
upstream Nova stylesheet and semantic tokens, and the exact same Geist files as
the native library. It currently configures button references; expand its explicit
loader list and installed dependencies as each component is ported. It is a
reproducible substitute for the upstream Next website's much larger registry/CLI
build, not a handwritten recreation of shadcn styles.

`compare` writes `side-by-side.png`, `difference.png` and `metrics.json`. Captures
are matched by dimensions, scale 1, theme and actual interaction state. Metrics
require visual review and do not automatically certify fidelity. `visual/button`
retains the current paired reference/native captures in both themes.

Read COMPONENTS.md for API selection, THEMING.md for overrides, and AGENTS.md for
agent usage and development rules. `ui::add_fonts` loads bundled Geist under OFL;
font and Lucide licence notices are retained. Shadcn's MIT notice is preserved in
reference/SHADCN-LICENSE. Source pins are in reference/upstream.json.
