//! Paint the complete pinned catalogue through Yakui's real draw-call batching.
use yakui::{Alignment, Rect, Vec2, Yakui};
use yakui_adequate_ui as ui;
#[test]
fn dense_lucide_catalogue_has_finite_geometry_and_valid_batched_indices() {
    let mut y = Yakui::new();
    let viewport = Vec2::new(1000., 800.);
    y.set_surface_size(viewport);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, viewport));
    y.start();
    yakui::align(Alignment::TOP_LEFT, || {
        ui::column(8., || {
            for row in ui::Icon::ALL.chunks(16) {
                ui::row(8., || {
                    for &icon in row {
                        ui::icon(icon, 24.);
                    }
                });
            }
        });
    });
    y.finish();
    let paint = y.paint();
    let mut calls = 0;
    for layer in paint.layers().iter() {
        for call in &layer.calls {
            calls += 1;
            assert!(call.vertices.len() <= u16::MAX as usize);
            assert!(call.vertices.iter().all(|v| v.position.is_finite()));
            assert!(
                call.indices
                    .iter()
                    .all(|&i| (i as usize) < call.vertices.len())
            );
        }
    }
    assert_eq!(calls, ui::Icon::ALL.len());
}
