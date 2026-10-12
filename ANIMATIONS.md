# Animations

Controls use a shared frame clock and retained, interruptible transitions. Logical
state changes immediately; rendering moves from its current value to the new
value. Reversing a switch or closing/reopening an overlay does not restart from an
endpoint. Durations are elapsed time, independent of frame rate.

Buttons, focus rings, switches, checkboxes, tabs and progress interpolate their
appearance. Accordions/collapsibles animate measured height; sheets, drawers,
sidebars and carousels move actual layout. Dialogs and popups retain their content
for exit fades. Skeletons pulse and `ui::spinner_auto(size)` rotates automatically.
The explicit `spinner(size, phase)` and `skeleton_at` APIs remain caller controlled.

## Host frame loop

The supplied gallery and consuming examples redraw continuously with vsync.
Games should declare UI each rendered frame. Hosts that sleep when idle can check
for active motion after layout has finished:

```rust
use yakui_adequate_ui as ui;
// Process input and application state changes first.
yak.start();
ui::button("Apply");
yak.finish();
let redraw = ui::animation::animations_active(&yak);
// Paint this frame; schedule another while redraw is true.
```

Application state changes still need a frame to declare their new targets. Timed
business events (toast expiration, carousel autoplay) and editor cursor blinking
need host wakeups even while no transition is active. `animations_active` reports
motion declared this frame, not a general event-loop scheduler.

## Custom transitions

Keep calls at stable Yakui call sites. The initial value renders settled.

```rust
use std::time::Duration;
use yakui_adequate_ui as ui;
use ui::animation::{animate, Easing, Motion};
let timing = Motion::new(Duration::from_millis(200), Easing::EaseOut);
animate(if expanded { 240. } else { 48. }, timing, |width| {
    ui::width(width, || ui::label("Navigation"));
});
```

`animate` accepts `f32`, `Vec2`, `Color`, `MotionStyle` and tuples of these.
`Transition<T>` exposes the same interpolation with caller-owned state and time.
`Easing::CubicBezier([x1, y1, x2, y2])` expects CSS timing coordinates with monotonic
x in [0, 1]. The predefined curves follow the pinned Tailwind timing functions.

```rust
use ui::animation::{motion, presence, Motion, MotionStyle};
presence(visible, Motion::default(), |sample| {
    if sample.present {
        motion(MotionStyle { opacity: sample.progress, ..Default::default() }, || {
            ui::card(|| ui::label("Saved"));
        });
    }
});
```

Presence invokes its callback even when closed so focus scopes can observe
closure. Keep the displayed data alive during exit. Closing content paints but
ignores pointer and keyboard input. Use the library's overlay APIs for focus
restoration and modal focus trapping; bare presence does not own focus.

`motion` applies opacity, translation, scale and clockwise rotation to a complete
paint subtree. Its transforms leave layout and hit bounds unchanged. Use an
animated width/height or an actual layout widget for large interactive movement.
Library sheets and carousel tracks already do this; popup zoom and small entry
translations use paint effects.

## Reduced motion and deterministic captures

```rust
ui::animation::with_motion_enabled(false, || {
    ui::button("Instant transitions");
});
ui::animation::with_animation_time(Duration::from_millis(50), || {
    // Declare the same component tree for deterministic sampling.
});
```

Static gallery screenshots settle motion by default. Explicit time samples an
actual trigger or key event:

```sh
cargo run --bin gallery -- --component dialog --state open --theme dark \
  --isolated --animation-time-ms 50 --screenshot /tmp/dialog.png
cargo run --bin gallery -- --component dialog --state open --keys Escape \
  --isolated --animation-time-ms 50 --screenshot /tmp/dialog-exit.png
cargo build --workspace
python3 tools/motion_capture.py --component switch --state default \
  --keys Tab,Space --theme dark --duration-ms 192
```

The sequence tool writes PNGs, timestamps and the Yakui pin. With ffmpeg installed
it also writes a video and contact sheet. Browser capture accepts an optional
elapsed animation time after its keyboard argument:

```sh
node tools/reference-app/capture.mjs switch-demo light /tmp/source.png \
  selected 600 400 '' 50
```

Pass `--reference-example switch-demo --state default --keys Tab,Space` to
`motion_capture.py`
to capture the matching browser frame and generate differences at every sample
(the reference Vite server must be running). Compare individual frames with the
existing `compare` binary. Reference timing is
sampled from Chromium's paused animation clock; the pinned component source is
unchanged. Use `--keys` or an actual `open`/hover trigger for native motion. `selected` is a
settled business-state preset, so it does not itself start a native transition.

## Current limits

This is tween-based motion, not a spring/physics engine. Carousel loop wrap travels
across the track rather than using a seamless duplicate-track wrap. Toast entry
and exit animate, but stack reflow and hover/drag physics remain immediate.
Line-style tab indicators do not slide between labels. Legacy response-only
tooltip shorthand has immediate visibility; the composed Tooltip API animates.
Rotated clips use axis-aligned bounding boxes. Typography and shadow differences
from browser rendering remain recorded in the catalogue.

The maintained Yakui fork supplies scoped paint effects and independent fragment
opacity for both WGPU and Vulkan. All Yakui dependencies in a consuming app must
use the same fork revision as this library; see Cargo.toml and reference/upstream.json.
Vulkan submission/completion contracts are documented in the fork's
`crates/yakui-vulkan/README.md`.

Slider hover and keyboard rings use the shared 150ms transition, independently
per thumb. Dragged values paint immediately; only the ring animates.
