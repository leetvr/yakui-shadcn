# Theming

Themes are ordinary Rust values, scoped to a Yakui subtree. There is no global
mutable theme or renderer dependency.

```rust
use yakui_adequate_ui as ui;
let mut theme = ui::Theme::dark();
theme.primary = ui::Color::hex(0x2563eb);
theme.primary_foreground = ui::Color::WHITE;
theme.radius = 8.0;
theme.font_family = "Geist".into();
ui::with_theme(theme, || {
    ui::button("Apply");
    ui::with_theme(ui::Theme::light(), || ui::label("Nested light surface"));
});
```

`Theme` contains the Neutral semantic colours (including charts and sidebar),
font family/size/line height, spacing unit, border width, and base radius. Default
light/dark values come from the pinned upstream OKLCH tokens. Typography and
component-specific sizes follow Nova. Some components still have explicit upstream
pixel dimensions; full dimension override coverage is tracked as unfinished work.

The radius scale is upstream's multiplicative scale: small 0.6, medium 0.8, large
1.0, extra large 1.4. Semantic translucent surfaces are composited in sRGB against
the enclosing surface so their colours match CSS on both linear game renderers.

Call `ui::add_fonts(&mut yak)` once to load Geist and the OFL Noto symbol fallback
for shortcuts such as ⌘ and ⇧. Font assets and source hashes are bundled. To use another family, load its font bytes into
Yakui's `font::Fonts` and set `font_family` to the font's internal family name.
Applications choose the renderer and UI scale through Yakui itself.

Motion can be disabled for a subtree without changing application state:

```rust
ui::animation::with_motion_enabled(false, || ui::button("Apply"));
```

Custom transitions accept ordinary `Motion` duration/easing values. See
ANIMATIONS.md for concrete examples and the vsync frame contract.
