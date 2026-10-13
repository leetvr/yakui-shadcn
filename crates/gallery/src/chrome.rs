//! Presentation owned by the gallery, separate from the components under test.
use crate::{Gallery, examples, showcases};
use std::time::Duration;
use yakui_shadcn as ui;

#[track_caller]
pub fn text(value: impl Into<String>, size: f32, color: ui::Color, weight: u16) {
    let mut style = ui::theme().text_style(size, color, weight);
    style.line_height_override = Some(size * 1.5);
    yakui::widgets::RenderText::with_style(value, style).show();
}
fn title(value: &str) -> String {
    let value = if value == "default" {
        "Overview"
    } else {
        value
    };
    let mut text = value.replace('-', " ");
    if let Some(first) = text.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    text
}
fn header(gallery: &mut Gallery) {
    let t = ui::theme();
    ui::fill_width(|| {
        ui::surface(t.background, None, 0., 20., || {
            yakui::widgets::List::row()
                .cross_axis_alignment(yakui::CrossAxisAlignment::Center)
                .item_spacing(12.)
                .show(|| {
                    text("shadcn/ui", 22., t.foreground, 600);
                    text(
                        "Unofficial Yakui port · Design by shadcn/ui",
                        13.,
                        t.muted_foreground,
                        400,
                    );
                    ui::spacer();
                    for (route, label) in [
                        (None, "Components"),
                        (Some("dashboard"), "Dashboard"),
                        (Some("settings"), "Settings"),
                        (Some("checkout"), "Checkout"),
                    ] {
                        if ui::Button::new(label)
                            .size(ui::ButtonSize::Sm)
                            .variant(if gallery.options.showcase.as_deref() == route {
                                ui::Variant::Secondary
                            } else {
                                ui::Variant::Ghost
                            })
                            .show()
                            .clicked
                        {
                            gallery.options.showcase = route.map(str::to_owned);
                        }
                    }
                    if ui::Button::new(if gallery.options.dark {
                        "Light"
                    } else {
                        "Dark"
                    })
                    .variant(ui::Variant::Outline)
                    .size(ui::ButtonSize::Sm)
                    .show()
                    .clicked
                    {
                        gallery.options.dark = !gallery.options.dark;
                    }
                });
        });
    });
    ui::separator(gallery.viewport.x);
}
pub fn render(gallery: &mut Gallery) {
    let t = ui::theme();
    ui::column(0., || {
        header(gallery);
        let height = (gallery.viewport.y - 74.).max(100.);
        yakui::widgets::List::row()
            .main_axis_size(yakui::MainAxisSize::Min)
            .cross_axis_alignment(yakui::CrossAxisAlignment::Start)
            .show(|| {
                let navigation = gallery.options.showcase.is_none() && gallery.viewport.x >= 800.;
                if navigation {
                    ui::width(208., || {
                        ui::surface(t.sidebar, None, 0., 20., || {
                            ui::scroll_area(height - 40., || {
                                ui::column(4., || {
                                    text("COMPONENTS", 10., t.muted_foreground, 600);
                                    for entry in &gallery.catalogue.components {
                                        if ui::Button::new(&entry.title)
                                            .size(ui::ButtonSize::Sm)
                                            .variant(if entry.name == gallery.options.component {
                                                ui::Variant::Secondary
                                            } else {
                                                ui::Variant::Ghost
                                            })
                                            .show()
                                            .clicked
                                        {
                                            gallery.options.component = entry.name.clone();
                                            gallery.options.variant = "default".into();
                                        }
                                    }
                                });
                            });
                        });
                    });
                }
                let width = (gallery.viewport.x - if navigation { 208. } else { 0. }).max(200.);
                let route = gallery
                    .options
                    .showcase
                    .clone()
                    .unwrap_or_else(|| gallery.options.component.clone());
                if gallery.route != route {
                    if route == "dashboard" {
                        gallery.showcase_state.enter_dashboard();
                    }
                    gallery.route = route;
                    gallery.page.snap(0.);
                    gallery.page.set_target(
                        1.,
                        ui::animation::Motion::new(
                            Duration::from_millis(220),
                            ui::animation::Easing::EaseOut,
                        ),
                        ui::animation::now(),
                    );
                }
                let opacity = if gallery.options.screenshot.is_some() {
                    1.
                } else {
                    gallery.page.value(ui::animation::now())
                };
                // Only opacity changes: controls keep their real layout and hit bounds,
                // and opening source panels never changes the preview's call site.
                ui::animation::motion(
                    ui::animation::MotionStyle {
                        opacity,
                        ..Default::default()
                    },
                    || {
                        ui::width(width, || {
                            page_scroll(&gallery.route.clone(), height, || {
                                ui::surface(t.background, None, 0., 32., || {
                                    let content_width = (width - 64.).max(160.);
                                    if let Some(name) = gallery.options.showcase.clone() {
                                        showcases::render(
                                            &name,
                                            &mut gallery.showcase_state,
                                            content_width,
                                        );
                                    } else {
                                        ui::width(content_width.min(1040.), || {
                                            component(gallery, content_width.min(1040.), navigation)
                                        });
                                    }
                                });
                            });
                        });
                    },
                );
            });
    });
}
fn component(gallery: &mut Gallery, width: f32, navigation: bool) {
    let t = ui::theme();
    let component = gallery.options.component.clone();
    ui::column(28., || {
        ui::column(6., || {
            text("COMPONENTS / YAKUI", 11., t.muted_foreground, 500);
            if let Some(entry) = gallery
                .catalogue
                .components
                .iter()
                .find(|e| e.name == component)
            {
                text(&entry.title, 32., t.foreground, 600);
                text(&entry.description, 15., t.muted_foreground, 400);
            }
        });
        if !navigation {
            ui::row(8., || {
                for (delta, label) in [(-1, "Previous"), (1, "Next")] {
                    if ui::Button::new(label)
                        .variant(ui::Variant::Outline)
                        .show()
                        .clicked
                    {
                        let pos = gallery
                            .catalogue
                            .components
                            .iter()
                            .position(|e| e.name == component)
                            .unwrap_or(0);
                        let pos = (pos as isize + delta)
                            .rem_euclid(gallery.catalogue.components.len() as isize)
                            as usize;
                        gallery.options.component = gallery.catalogue.components[pos].name.clone();
                    }
                }
            });
        }
        for variant in examples::variants(&component) {
            let key = format!("{component}:{variant}");
            let state = gallery.states.entry(key.clone()).or_default();
            ui::column(10., || {
                text(title(variant), 16., t.foreground, 600);
                ui::fill_width(|| {
                    ui::surface(t.background, Some(t.border), 10., 1., || {
                        ui::column(0., || {
                            ui::fill_width(|| {
                                ui::surface(t.background, None, 10., 28., || {
                                    ui::min_size(ui::Vec2::new(0., 144.), || {
                                        examples::render(&component, variant, state, false)
                                    });
                                });
                            });
                            ui::separator(width - 2.);
                            ui::fill_width(|| {
                                ui::surface(t.background, None, 10., 16., || {
                                    gallery.panels.entry(key).or_default().show(
                                        &component,
                                        variant,
                                        width - 34.,
                                    );
                                });
                            });
                        });
                    });
                });
                if state.count > 0 {
                    text(
                        format!("{} actions", state.count),
                        12.,
                        t.muted_foreground,
                        400,
                    );
                }
            });
        }
    });
}

/// Reset only the page's scroll offset when its route changes. The descendant
/// widgets remain mounted, so a code disclosure does not reset a live control.
#[derive(Debug)]
struct PageScroll {
    route: String,
    inner: yakui::widgets::ScrollableWidget,
}
impl yakui::widget::Widget for PageScroll {
    type Props<'a> = &'a str;
    type Response = ();
    fn new() -> Self {
        Self {
            route: String::new(),
            inner: yakui::widgets::ScrollableWidget::new(),
        }
    }
    fn update(&mut self, route: &str) {
        if self.route != route {
            self.route = route.to_owned();
            self.inner = yakui::widgets::ScrollableWidget::new();
        }
        self.inner.update(yakui::widgets::Scrollable::vertical());
    }
    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        constraints: yakui::Constraints,
    ) -> ui::Vec2 {
        self.inner.layout(ctx, constraints)
    }
    fn paint(&self, ctx: yakui::widget::PaintContext<'_>) {
        self.inner.paint(ctx);
    }
    fn event_interest(&self) -> yakui::event::EventInterest {
        self.inner.event_interest()
    }
    fn event(
        &mut self,
        ctx: yakui::widget::EventContext<'_>,
        event: &yakui::event::WidgetEvent,
    ) -> yakui::event::EventResponse {
        self.inner.event(ctx, event)
    }
}
fn page_scroll(route: &str, height: f32, children: impl FnOnce()) {
    yakui::widgets::ConstrainedBox::new(yakui::Constraints {
        min: ui::Vec2::ZERO,
        max: ui::Vec2::new(f32::INFINITY, height),
    })
    .show(|| {
        yakui::util::widget_children::<PageScroll, _>(children, route);
    });
}
