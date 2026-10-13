//! Application-sized compositions use the same public API as a consuming program.
//! Checkout follows the pinned Base Field example; dashboard follows the upstream
//! cards/chart/table arrangement. Business actions remain local demo state.
use std::time::Duration;
use ui::animation::{Easing, Motion, MotionStyle};
use ui::{Button, Variant};
use yakui_shadcn as ui;
pub const NAMES: &[&str] = &["dashboard", "settings", "checkout"];
#[derive(Default)]
pub struct State {
    table: ui::DataTableState,
    dashboard_running: bool,
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
    let width = available_width.min(1100.);
    ui::width(width, || match name {
        "dashboard" => dashboard(state, width),
        "settings" => settings(state, width),
        "checkout" => checkout(state, width),
        _ => unreachable!(),
    });
}
impl State {
    pub fn enter_dashboard(&mut self) {
        self.dashboard_running = false;
    }
}
fn dashboard(state: &mut State, width: f32) {
    // Reset only the tween for one frame. The children remain at the same call
    // sites, so replay doesn't replace chart focus or the table's retained input.
    let running = std::mem::replace(&mut state.dashboard_running, true);
    ui::animation::animate(
        if running { 1. } else { 0. },
        Motion::new(
            Duration::from_millis(if running { 1100 } else { 0 }),
            Easing::Linear,
        ),
        |progress| dashboard_content(state, width, progress * 1100.),
    );
}
fn phase(time: f32, delay: f32, duration: f32) -> f32 {
    Easing::EaseOut.sample(((time - delay) / duration).clamp(0., 1.))
}
#[track_caller]
fn entrance(progress: f32, children: impl FnOnce()) {
    ui::animation::motion(
        MotionStyle {
            opacity: progress,
            translation: ui::Vec2::new(0., 10. * (1. - progress)),
            ..Default::default()
        },
        children,
    );
}
fn grouped(value: u32) -> String {
    if value >= 1000 {
        format!("{},{:03}", value / 1000, value % 1000)
    } else {
        value.to_string()
    }
}
fn metric_value(index: usize, progress: f32) -> String {
    match index {
        0 => {
            let cents = (125000. * progress).round() as u32;
            format!("${}.{:02}", grouped(cents / 100), cents % 100)
        }
        1 => grouped((1234. * progress).round() as u32),
        2 => grouped((45678. * progress).round() as u32),
        _ => format!("{:.1}%", 4.5 * progress),
    }
}
#[derive(Debug)]
struct ChartReveal(f32);
impl yakui::widget::Widget for ChartReveal {
    type Props<'a> = f32;
    type Response = ();
    fn new() -> Self {
        Self(0.)
    }
    fn update(&mut self, progress: f32) {
        self.0 = progress;
    }
    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        constraints: yakui::Constraints,
    ) -> ui::Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        // Lay out the complete chart before shrinking its clip. Its points,
        // scale, labels and hit coordinates never depend on animation progress.
        let size = self.default_layout(ctx, constraints);
        ui::Vec2::new(size.x * self.0, size.y)
    }
}
fn dashboard_content(state: &mut State, width: f32, time: f32) {
    ui::column(24., || {
        crate::chrome::responsive_row(width < 600., 12., || {
            ui::heading(2, "Dashboard");
            if Button::new("Replay animation")
                .variant(Variant::Outline)
                .show()
                .clicked
            {
                state.dashboard_running = false;
            }
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
                metric_value(0, phase(time, 0., 700.)),
                "+12.5%",
                "Trending up this month",
            ),
            (
                "New Customers",
                metric_value(1, phase(time, 70., 700.)),
                "−20%",
                "Acquisition needs attention",
            ),
            (
                "Active Accounts",
                metric_value(2, phase(time, 140., 700.)),
                "+12.5%",
                "Strong user retention",
            ),
            (
                "Growth Rate",
                metric_value(3, phase(time, 210., 700.)),
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
        for (group_index, group) in metrics.chunks(columns).enumerate() {
            ui::row(16., || {
                for (index, (title, value, delta, note)) in group.iter().enumerate() {
                    let progress = phase(time, (group_index * columns + index) as f32 * 70., 360.);
                    entrance(progress, || {
                        ui::width(
                            (width - 16. * (columns - 1) as f32) / columns as f32,
                            || {
                                ui::card(|| {
                                    ui::card_header(|| {
                                        ui::card_description(*title);
                                        ui::heading(2, value.as_str());
                                    });
                                    ui::card_content(|| {
                                        ui::column(8., || {
                                            ui::badge(*delta, Variant::Outline);
                                            ui::muted(*note);
                                        })
                                    });
                                });
                            },
                        );
                    });
                }
            });
        }
        entrance(phase(time, 160., 400.), || {
            ui::card(|| {
                ui::card_header(|| {
                    ui::card_title("Total Visitors");
                    ui::card_description("Visitors for the last 3 months");
                });
                ui::card_content(|| {
                    let source: Vec<serde_json::Value> =
                        serde_json::from_str(include_str!("../../../reference/chart-demo.json"))
                            .unwrap();
                    let chart_width = (width - 32.).max(200.);
                    ui::width(chart_width, || {
                        yakui::align(yakui::Alignment::TOP_LEFT, || {
                            yakui::util::widget_children::<ChartReveal, _>(
                                || {
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
                                    .size(ui::Vec2::new(chart_width, 220.))
                                    .show();
                                },
                                phase(time, 200., 800.),
                            );
                        });
                    });
                });
            });
        });
        entrance(phase(time, 350., 400.), || {
            ui::column(16., || {
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
                crate::chrome::horizontal_scroll(|| {
                    ui::width(width.max(600.), || {
                        ui::data_table(&columns, &rows, &mut state.table)
                    });
                });
            });
        });
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
fn checkout(state: &mut State, width: f32) {
    let width = width.min(448.);
    ui::width(width, || {
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
            crate::chrome::responsive_row(width < 400., 16., || {
                ui::width(
                    if width < 400. {
                        width
                    } else {
                        (width - 32.) / 3.
                    },
                    || {
                        ui::field("Month", "", None, || {
                            ui::select(&mut state.month, &MONTHS, "MM", false)
                        })
                    },
                );
                ui::width(
                    if width < 400. {
                        width
                    } else {
                        (width - 32.) / 3.
                    },
                    || {
                        ui::field("Year", "", None, || {
                            ui::select(&mut state.year, &YEARS, "YYYY", false)
                        })
                    },
                );
                ui::width(
                    if width < 400. {
                        width
                    } else {
                        (width - 32.) / 3.
                    },
                    || {
                        ui::field("CVV", "", None, || {
                            ui::input(&mut state.cvv, "123");
                        })
                    },
                );
            });
            ui::separator(width);
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

#[cfg(test)]
mod dashboard_tests {
    use super::*;
    use yakui::{Rect, Vec2, WidgetId, Yakui, event::Event, input::MouseButton};

    fn find(yak: &Yakui, id: WidgetId, kind: &str, out: &mut Vec<WidgetId>) {
        let node = yak.dom().get(id).unwrap();
        if node.widget.type_name().ends_with(kind) {
            out.push(id);
        }
        for &child in &node.children {
            find(yak, child, kind, out);
        }
    }
    fn widgets(yak: &Yakui, kind: &str) -> Vec<WidgetId> {
        let mut out = vec![];
        find(yak, yak.dom().root(), kind, &mut out);
        out
    }
    fn rect(yak: &Yakui, id: WidgetId) -> Rect {
        yak.layout_dom().get(id).unwrap().rect
    }
    fn frame(yak: &mut Yakui, state: &mut State, ms: u64, enabled: bool) {
        yak.start();
        ui::animation::with_animation_time(Duration::from_millis(ms), || {
            ui::animation::with_motion_enabled(enabled, || {
                render("dashboard", state, 1100.);
            });
        });
        yak.finish();
        yak.paint();
    }
    fn click(yak: &mut Yakui, id: WidgetId) {
        yak.handle_event(Event::CursorMoved(Some(rect(yak, id).center())));
        for down in [true, false] {
            yak.handle_event(Event::MouseButtonChanged {
                button: MouseButton::One,
                down,
            });
        }
    }
    #[test]
    fn dashboard_replay_preserves_chart_layout_and_actual_table_selection() {
        let mut yak = crate::new_yak();
        yak.set_surface_size(Vec2::new(1200., 1200.));
        yak.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, yak.surface_size()));
        let mut state = State::default();
        frame(&mut yak, &mut state, 0, true);
        frame(&mut yak, &mut state, 0, true);
        let plot = widgets(&yak, "::PlotWidget")[0];
        let reveal = widgets(&yak, "::ChartReveal")[0];
        let plot_rect = rect(&yak, plot);
        assert_eq!(rect(&yak, reveal).size().x, 0.);
        frame(&mut yak, &mut state, 500, true);
        assert_eq!(rect(&yak, plot), plot_rect);
        assert!(rect(&yak, reveal).size().x > 0.);
        assert!(rect(&yak, reveal).size().x < plot_rect.size().x);
        frame(&mut yak, &mut state, 1200, true);
        assert_eq!(rect(&yak, reveal).size().x, plot_rect.size().x);
        assert!(!ui::animation::animations_active(&yak));
        let controls = widgets(&yak, "::ControlWidget");
        let checkbox = controls
            .iter()
            .copied()
            .find(|&id| rect(&yak, id).size() == Vec2::splat(16.))
            .unwrap();
        click(&mut yak, checkbox);
        frame(&mut yak, &mut state, 1200, true);
        assert_eq!(state.table.selected.len(), 5);
        // The first control is the real replay button. Repeated clicks during
        // motion restart the same tween without remounting any controls.
        click(&mut yak, controls[0]);
        frame(&mut yak, &mut state, 1300, true);
        frame(&mut yak, &mut state, 1300, true);
        frame(&mut yak, &mut state, 1300, true);
        assert_eq!(rect(&yak, reveal).size().x, 0.);
        frame(&mut yak, &mut state, 1700, true);
        click(&mut yak, controls[0]);
        frame(&mut yak, &mut state, 1700, true);
        frame(&mut yak, &mut state, 1700, true);
        frame(&mut yak, &mut state, 1700, true);
        assert_eq!(rect(&yak, reveal).size().x, 0.);
        assert_eq!(widgets(&yak, "::ControlWidget"), controls);
        assert_eq!(widgets(&yak, "::PlotWidget"), vec![plot]);
        assert_eq!(rect(&yak, plot), plot_rect);
        assert_eq!(state.table.selected.len(), 5);
        frame(&mut yak, &mut state, 1800, false);
        assert_eq!(rect(&yak, reveal).size().x, plot_rect.size().x);
        assert!(!ui::animation::animations_active(&yak));
    }
}
