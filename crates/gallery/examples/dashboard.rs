//! Run: cargo run -p adequate-gallery --example dashboard
mod support;
use yakui_adequate_ui as ui;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut table = ui::DataTableState::default();
    support::run(move || {
        ui::width(740., || {
            ui::column(24., || {
                ui::heading(2, "Dashboard");
                ui::card(|| {
                    ui::card_header(|| {
                        ui::card_title("Visitors");
                        ui::card_description("This week");
                    });
                    ui::card_content(|| {
                        ui::Chart::new(
                            [12., 18., 9., 26., 32.]
                                .iter()
                                .enumerate()
                                .map(|(i, n)| {
                                    ui::ChartPoint::new(format!("Day {}", i + 1), vec![*n])
                                })
                                .collect(),
                            vec![ui::ChartSeries::new("Visitors", ui::theme().chart[0])],
                        )
                        .kind(ui::ChartKind::Line)
                        .size(ui::Vec2::new(680., 180.))
                        .show();
                    });
                });
                ui::data_table(
                    &[
                        ui::TableColumn::new("Status", 140.),
                        ui::TableColumn::new("Email", 300.),
                        ui::TableColumn::new("Amount", 120.).numeric(),
                    ],
                    &[
                        ui::TableRow::new(1, ["success", "m@example.com", "$316.00"]),
                        ui::TableRow::new(2, ["processing", "p@example.com", "$242.00"]),
                    ],
                    &mut table,
                );
            })
        })
    })
}
