//! Run: cargo run -p adequate-gallery --example settings
mod support;
use yakui_adequate_ui as ui;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut name = String::from("Kane");
    let mut vsync = true;
    let mut quality = 1;
    let mut applied = false;
    support::run(move || {
        ui::width(420., || {
            ui::card(|| {
                ui::card_header(|| {
                    ui::card_title("Graphics");
                    ui::card_description("Display settings");
                });
                ui::card_content(|| {
                    ui::form(|| {
                        ui::field(
                            "Profile name",
                            "These preferences belong to this profile.",
                            None,
                            || {
                                ui::input(&mut name, "Name");
                            },
                        );
                        ui::checkbox("VSync", &mut vsync);
                        ui::radio_group(&["Low", "Medium", "High"], &mut quality);
                        if applied {
                            ui::alert("Applied", "Your settings have been saved in memory.", false);
                        }
                    })
                });
                ui::card_footer(|| {
                    if ui::button("Apply").clicked {
                        applied = true;
                    }
                });
            })
        })
    })
}
