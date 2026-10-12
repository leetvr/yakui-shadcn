//! Shape actual bundled faces through the same font system used by widgets.
use yakui_adequate_ui as ui;
#[test]
fn bundled_fonts_shape_english_shortcut_symbols_without_missing_glyphs() {
    let mut y = yakui::Yakui::new();
    ui::add_fonts(&mut y);
    let fonts = y.dom().get_global_or_init(yakui::font::Fonts::default);
    fonts.with_system(|system| {
        let style = ui::Theme::light().text_style(14., ui::Color::BLACK, 400);
        let mut buffer = yakui::cosmic_text::Buffer::new(system, style.to_metrics(1.));
        for text in ["⇧⌘P ⌥", "Save ⌘S", "New Team ⌘+T"] {
            buffer.set_text(
                system,
                text,
                &style.attrs.as_attrs(),
                yakui::cosmic_text::Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(system, true);
            let glyphs: Vec<_> = buffer
                .layout_runs()
                .flat_map(|run| run.glyphs.iter())
                .collect();
            assert!(!glyphs.is_empty());
            assert!(
                glyphs.iter().all(|glyph| glyph.glyph_id != 0),
                "Missing glyph in {text}"
            );
            assert!(
                glyphs
                    .iter()
                    .all(|glyph| glyph.x.is_finite() && glyph.w.is_finite())
            );
        }
    });
}
