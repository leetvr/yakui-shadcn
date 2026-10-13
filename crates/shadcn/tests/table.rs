//! Layout and scrolling of actual public table slots and live controls.
use yakui::{Alignment, Rect, Vec2, WidgetId, Yakui, event::Event};
use yakui_shadcn as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    y.set_surface_size(Vec2::new(600., 400.));
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, Vec2::new(600., 400.)));
    ui::add_fonts(&mut y);
    y
}
fn frame<T>(y: &mut Yakui, body: impl FnOnce() -> T) -> T {
    y.start();
    let mut result = None;
    yakui::align(Alignment::TOP_LEFT, || result = Some(body()));
    y.finish();
    result.unwrap()
}
fn rects(y: &Yakui, suffix: &str) -> Vec<Rect> {
    fn visit(y: &Yakui, id: WidgetId, suffix: &str, result: &mut Vec<Rect>) {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with(suffix) {
            result.push(y.layout_dom().get(id).unwrap().rect);
        }
        for &child in &node.children {
            visit(y, child, suffix, result);
        }
    }
    let mut result = Vec::new();
    visit(y, y.dom().root(), suffix, &mut result);
    result
}
#[test]
fn table_slots_align_columns_spans_and_footer_without_stretching_row_content() {
    let mut y = yak();
    let mut amount = ui::ControlResponse::default();
    frame(&mut y, || {
        ui::Table::new(&[100., 150., 150.]).show(|| {
            ui::table_header(|| {
                ui::table_row(false, || {
                    for heading in ["Invoice", "Status", "Amount"] {
                        ui::table_head(heading);
                    }
                })
            });
            ui::table_body(|| {
                for selected in [true, false] {
                    ui::table_row(selected, || {
                        ui::table_cell(|| ui::table_cell_text("INV001", true));
                        ui::table_cell(|| ui::text("Paid"));
                        ui::table_cell(|| ui::text("$250.00"));
                    });
                }
            });
            ui::table_footer(|| {
                ui::table_row(false, || {
                    ui::TableCell::new()
                        .span(2)
                        .show(|| ui::table_cell_text("Total", false));
                    ui::TableCell::new()
                        .align(ui::TableCellAlign::End)
                        .show(|| {
                            amount = ui::button("Apply");
                        });
                })
            });
        })
    });
    let rows = rects(&y, "::TableRowWidget");
    assert_eq!(
        rows.iter().map(|r| r.size().y).collect::<Vec<_>>(),
        [40., 37., 36., 49.]
    );
    assert!(rows.iter().all(|r| r.size().x == 400.));
    let cells = rects(&y, "::CellWidget");
    assert_eq!(cells[0].pos().x, cells[3].pos().x);
    assert_eq!(cells[1].pos().x, cells[4].pos().x);
    assert_eq!(cells[9].size().x, 250.);
    let button = y.layout_dom().get(amount.id.unwrap()).unwrap().rect;
    assert_eq!(button.max().x, rows[3].max().x - 8.);
    assert_eq!(button.size().y, 32.);
}
fn wide_table() -> [ui::ControlResponse; 2] {
    let mut responses = [ui::ControlResponse::default(); 2];
    ui::width(200., || {
        ui::Table::new(&[180., 180.]).show(|| {
            ui::table_body(|| {
                ui::table_row(false, || {
                    ui::table_cell(|| {
                        responses[0] = ui::button("First");
                    });
                    ui::TableCell::new()
                        .align(ui::TableCellAlign::End)
                        .show(|| {
                            responses[1] = ui::button("Last");
                        });
                })
            })
        })
    });
    responses
}
#[test]
fn table_scrolls_horizontally_and_reveals_controls_when_focus_changes() {
    let mut y = yak();
    let responses = frame(&mut y, wide_table);
    let viewport = rects(&y, "::TableViewport")[0];
    let before = y.layout_dom().get(responses[1].id.unwrap()).unwrap().rect;
    assert_eq!(viewport.size().x, 200.);
    assert!(before.max().x > viewport.max().x);
    y.handle_event(Event::CursorMoved(Some(viewport.center())));
    y.handle_event(Event::MouseScroll {
        delta: Vec2::new(100., 0.),
    });
    frame(&mut y, wide_table);
    let scrolled = y.layout_dom().get(responses[1].id.unwrap()).unwrap().rect;
    assert!((scrolled.pos().x - (before.pos().x - 100.)).abs() < 0.001);
    y.request_focus(responses[1].id);
    frame(&mut y, wide_table);
    let focused = y.layout_dom().get(responses[1].id.unwrap()).unwrap().rect;
    assert!(focused.max().x <= viewport.max().x);
    y.request_focus(responses[0].id);
    frame(&mut y, wide_table);
    let first = y.layout_dom().get(responses[0].id.unwrap()).unwrap().rect;
    assert!(first.pos().x >= viewport.pos().x);
}
