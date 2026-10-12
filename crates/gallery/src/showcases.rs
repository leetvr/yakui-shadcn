//! Application-sized compositions use the same public API as a consuming program.
//! Checkout follows the pinned Base Field example; dashboard follows the upstream
//! cards/chart/table arrangement. Business actions remain local demo state.
use ui::{Button, Variant};
use yakui_adequate_ui as ui;
pub const NAMES: &[&str] = &["dashboard", "settings", "checkout"];
#[derive(Default)]
pub struct State {
    table: ui::DataTableState,
    tab: usize,
    name: String,
    email: String,
    bio: String,
    notifications: bool,
    security: bool,
    density: usize,
    card_number: String,
    cvv: String,
    month: ui::SelectState,
    year: ui::SelectState,
    same_address: bool,
    comments: String,
    submitted: bool,
    toasts: ui::ToastManager,
}
fn saved(state: &mut State) {
    state.toasts.add(
        "Changes saved",
        "Your preferences have been updated.",
        ui::ToastKind::Success,
    );
}
pub fn render(name: &str, state: &mut State, available_width: f32) {
    let width = available_width.clamp(280., 1100.);
    ui::width(width, || match name {
        "dashboard" => dashboard(state, width),
        "settings" => settings(state, width),
        "checkout" => checkout(state),
        _ => unreachable!(),
    });
}
fn dashboard(state: &mut State, width: f32) {
    ui::column(24., || {
        ui::row(16., || {
            ui::heading(2, "Dashboard");
            if Button::new("Download report")
                .variant(Variant::Outline)
                .show()
                .clicked
            {
                state.toasts.add(
                    "Report ready",
                    "This demonstration keeps its data in memory.",
                    ui::ToastKind::Info,
                );
            }
        });
        let metrics = [
            (
                "Total Revenue",
                "$1,250.00",
                "+12.5%",
                "Trending up this month",
            ),
            (
                "New Customers",
                "1,234",
                "−20%",
                "Acquisition needs attention",
            ),
            (
                "Active Accounts",
                "45,678",
                "+12.5%",
                "Strong user retention",
            ),
            (
                "Growth Rate",
                "4.5%",
                "+4.5%",
                "Steady performance increase",
            ),
        ];
        let columns = if width >= 900. {
            4
        } else if width >= 550. {
            2
        } else {
            1
        };
        for group in metrics.chunks(columns) {
            ui::row(16., || {
                for &(title, value, delta, note) in group {
                    ui::width(
                        (width - 16. * (columns - 1) as f32) / columns as f32,
                        || {
                            ui::card(|| {
                                ui::card_header(|| {
                                    ui::card_description(title);
                                    ui::heading(2, value);
                                });
                                ui::card_content(|| {
                                    ui::column(8., || {
                                        ui::badge(delta, Variant::Outline);
                                        ui::muted(note);
                                    })
                                });
                            });
                        },
                    );
                }
            });
        }
        ui::card(|| {
            ui::card_header(|| {
                ui::card_title("Total Visitors");
                ui::card_description("Visitors for the last 3 months");
            });
            ui::card_content(|| {
                let source: Vec<serde_json::Value> =
                    serde_json::from_str(include_str!("../../../reference/chart-demo.json"))
                        .unwrap();
                ui::Chart::new(
                    source
                        .iter()
                        .map(|p| {
                            ui::ChartPoint::new(
                                p["date"].as_str().unwrap(),
                                vec![
                                    p["desktop"].as_f64().unwrap(),
                                    p["mobile"].as_f64().unwrap(),
                                ],
                            )
                        })
                        .collect(),
                    vec![
                        ui::ChartSeries::new("Desktop", ui::theme().chart[0]),
                        ui::ChartSeries::new("Mobile", ui::theme().chart[1]),
                    ],
                )
                .kind(ui::ChartKind::Area)
                .stacked(true)
                .size(ui::Vec2::new((width - 32.).max(200.), 220.))
                .show();
            });
        });
        ui::heading(3, "Recent payments");
        let columns = [
            ui::TableColumn::new("Status", (width * 0.18).max(80.)),
            ui::TableColumn::new("Email", (width * 0.43).max(160.)),
            ui::TableColumn::new("Amount", (width * 0.18).max(80.)).numeric(),
        ];
        let rows = vec![
            ui::TableRow::new(1, ["success", "m@example.com", "$316.00"]),
            ui::TableRow::new(2, ["success", "p@example.com", "$242.00"]),
            ui::TableRow::new(3, ["processing", "ken99@example.com", "$837.00"]),
            ui::TableRow::new(4, ["success", "abe45@example.com", "$874.00"]),
            ui::TableRow::new(5, ["failed", "carmella@example.com", "$721.00"]),
        ];
        ui::data_table(&columns, &rows, &mut state.table);
    });
}
fn settings(state: &mut State, width: f32) {
    ui::column(24., || {
        ui::heading(2, "Settings");
        ui::muted("Manage your account settings and email preferences.");
        ui::separator(width);
        ui::tabs(
            &["Profile", "Notifications", "Appearance"],
            &mut state.tab,
            ui::TabsVariant::Default,
            |tab| {
                ui::width(width.min(600.), || {
                    ui::form(|| match tab {
                        0 => {
                            ui::heading(3, "Profile");
                            ui::field(
                                "Username",
                                "This is your public display name.",
                                None,
                                || {
                                    ui::input(&mut state.name, "shadcn");
                                },
                            );
                            ui::field(
                                "Email",
                                "You can manage verified email addresses in your account.",
                                None,
                                || {
                                    ui::input(&mut state.email, "m@example.com");
                                },
                            );
                            ui::field("Bio", "Tell us a little bit about yourself.", None, || {
                                ui::textarea(&mut state.bio, "I own a computer.");
                            });
                        }
                        1 => {
                            ui::heading(3, "Notifications");
                            ui::switch("Communication emails", &mut state.notifications, false);
                            ui::muted("Receive emails about your account activity.");
                            ui::switch("Security emails", &mut state.security, false);
                            ui::muted("Receive emails about your account security.");
                        }
                        _ => {
                            ui::heading(3, "Appearance");
                            ui::muted("Select your preferred display density.");
                            ui::radio_group(
                                &["Default", "Comfortable", "Compact"],
                                &mut state.density,
                            );
                        }
                    })
                });
            },
        );
        if ui::button("Update preferences").clicked {
            saved(state);
        }
    });
}
fn checkout(state: &mut State) {
    ui::width(448., || {
        ui::form(|| {
            ui::heading(3, "Payment Method");
            ui::muted("All transactions are secure and encrypted");
            ui::field(
                "Name on Card",
                "",
                if state.submitted && state.name.trim().is_empty() {
                    Some("Enter the name on your card.")
                } else {
                    None
                },
                || {
                    ui::input(&mut state.name, "Evil Rabbit");
                },
            );
            ui::field(
                "Card Number",
                "Enter your 16-digit card number",
                if state.submitted
                    && state
                        .card_number
                        .chars()
                        .filter(char::is_ascii_digit)
                        .count()
                        != 16
                {
                    Some("Enter a 16-digit card number.")
                } else {
                    None
                },
                || {
                    ui::input(&mut state.card_number, "1234 5678 9012 3456");
                },
            );
            ui::row(16., || {
                ui::width(138., || {
                    ui::field("Month", "", None, || {
                        ui::select(&mut state.month, &MONTHS, "MM", false)
                    })
                });
                ui::width(138., || {
                    ui::field("Year", "", None, || {
                        ui::select(&mut state.year, &YEARS, "YYYY", false)
                    })
                });
                ui::width(138., || {
                    ui::field("CVV", "", None, || {
                        ui::input(&mut state.cvv, "123");
                    })
                });
            });
            ui::separator(448.);
            ui::heading(3, "Billing Address");
            ui::muted("The billing address associated with your payment method");
            ui::checkbox("Same as shipping address", &mut state.same_address);
            ui::field("Comments", "", None, || {
                ui::textarea(&mut state.comments, "Add any additional comments");
            });
            ui::row(8., || {
                if ui::button("Submit").clicked {
                    state.submitted = true;
                    let valid = !state.name.trim().is_empty()
                        && state
                            .card_number
                            .chars()
                            .filter(char::is_ascii_digit)
                            .count()
                            == 16
                        && state.cvv.chars().filter(char::is_ascii_digit).count() == 3
                        && state.month.selected.is_some()
                        && state.year.selected.is_some();
                    if valid {
                        state.toasts.add(
                            "Form submitted",
                            "This local demonstration makes no payment.",
                            ui::ToastKind::Success,
                        );
                    } else {
                        state.toasts.add(
                            "Check the form",
                            "Supply a name, card number, expiry and CVV.",
                            ui::ToastKind::Error,
                        );
                    }
                }
                if Button::new("Cancel")
                    .variant(Variant::Outline)
                    .show()
                    .clicked
                {
                    *state = State::default();
                }
            });
        })
    });
}
const MONTHS: [&str; 12] = [
    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12",
];
const YEARS: [&str; 6] = ["2024", "2025", "2026", "2027", "2028", "2029"];
pub fn overlays(name: &str, state: &mut State) {
    if name == "checkout" {
        ui::select_content(&mut state.month, &MONTHS);
        ui::select_content(&mut state.year, &YEARS);
    }
    ui::toast_viewport(&mut state.toasts);
}
