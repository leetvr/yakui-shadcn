# Building interfaces with yakui-shadcn

This is an unofficial port of shadcn/ui to Yakui. Credit shadcn/ui and its
contributors for the component design and examples, and Base UI for interaction
behavior. Describe this project as a Rust/Yakui implementation, not an original
UI design system. Preserve upstream attribution and license notices.

Use `yakui-shadcn as ui`. Prefer existing components before custom widgets.
Read README.md and the source API documentation. State belongs to your application: pass mutable
values into controls and act on `ControlResponse.clicked`. Use scoped theme tokens. Put multiple sequential children in `ui::row` or
`ui::column`; a generic Yakui container overlays its children.

```rust
use yakui_shadcn as ui;
ui::card(|| {
    ui::card_header(|| {
        ui::card_title("Graphics");
        ui::card_description("Display settings");
    });
    ui::card_content(|| { ui::checkbox("VSync", &mut settings.vsync); });
    ui::card_footer(|| { if ui::button("Apply").clicked { apply_settings(); } });
});
```

Call `ui::add_fonts(&mut yak)` once before the first frame. Declare components
between `yak.start()` and `yak.finish()`, at stable call sites. Yakui resets retained
widget interaction state when their call sites change.

Render overlays at the application root, after ordinary content, outside scroll
areas. Keep `OverlayState` in application state. Open with the actual trigger's
response so dismissal can return focus. Text editing uses Yakui's Cosmic Text editor.
The public library depends on no renderer; games can paint it with Yakui/Vulkan.

Use existing motion primitives before custom timers. Retain stable component call
sites and keep overlay content data alive through its exit transition. Target games
that declare and paint UI every vsync frame; do not add idle-host scheduling or
non-vsync UI support. Consume
the same pinned Yakui fork for every Yakui crate to avoid duplicate UI contexts.

# Working on this repository

The target is English interfaces for internal work and prototypes. Prioritize
useful component coverage and common interactions; avoid exhaustive edge-case
work and browser-parity certification. Consult reference/components.json;
do not infer completion from a public function or a gallery screenshot. Study pinned
Base UI source, Nova styles and actual examples before implementing a component.
Use the gallery and real input cycles; do not substitute static imagery.

Keep the architecture simple. YAGNI. Prefer composition and ordinary Rust. Document
complex state ownership/order with narrative comments. Never run Clippy or add
mocked/fixture integration tests. Do not interact with Kane's phone. Mobile visual
validation belongs to Kane. The project brief authorizes desktop visual verification.

The agreed completion target is a usable full catalogue, common examples and
explicit gaps. All 461 documentation demos are reference material; do not expand
work toward exhaustive preview parity. Fix material normal-use problems and keep
minor polish/rare cases documented. Commit meaningful verified increments.
Do not equate prototype readiness with exhaustive upstream parity certification.
