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
    let compact = gallery.viewport.x < 800.;
    ui::fill_width(|| {
        ui::surface(
            t.background,
            None,
            0.,
            if compact { 12. } else { 20. },
            || {
                ui::column(8., || {
                    yakui::widgets::List::row()
                        .cross_axis_alignment(yakui::CrossAxisAlignment::Center)
                        .item_spacing(12.)
                        .show(|| {
                            text("shadcn/ui", 22., t.foreground, 600);
                            if !compact {
                                text(
                                    "Unofficial Yakui port · Design by shadcn/ui",
                                    13.,
                                    t.muted_foreground,
                                    400,
                                );
                            }
                            ui::spacer();
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
                    horizontal_scroll(|| {
                        ui::row(4., || {
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
                        });
                    });
                    if compact {
                        text(
                            "Unofficial Yakui port · Design by shadcn/ui",
                            11.,
                            t.muted_foreground,
                            400,
                        );
                    }
                });
            },
        );
    });
    ui::separator(gallery.viewport.x);
}

// Changing a List's direction retains the same descendants and their input state.
pub(crate) fn responsive_row(stacked: bool, gap: f32, children: impl FnOnce()) {
    let list = if stacked {
        yakui::widgets::List::column()
    } else {
        yakui::widgets::List::row()
    };
    list.main_axis_size(yakui::MainAxisSize::Min)
        .item_spacing(gap)
        .show(children);
}

pub fn render(gallery: &mut Gallery) {
    let t = ui::theme();
    yakui::util::widget_children::<GalleryFrame, _>(
        || {
            ui::column(0., || header(gallery));
            let compact = gallery.viewport.x < 800.;
            let height = gallery.viewport.y;
            yakui::widgets::List::row()
                .main_axis_size(yakui::MainAxisSize::Min)
                .cross_axis_alignment(yakui::CrossAxisAlignment::Start)
                .show(|| {
                    let navigation =
                        gallery.options.showcase.is_none() && gallery.viewport.x >= 800.;
                    if navigation {
                        ui::width(208., || {
                            ui::surface(t.sidebar, None, 0., 20., || {
                                ui::scroll_area(height - 40., || {
                                    ui::column(4., || {
                                        text("COMPONENTS", 10., t.muted_foreground, 600);
                                        for entry in &gallery.catalogue.components {
                                            if ui::Button::new(&entry.title)
                                                .size(ui::ButtonSize::Sm)
                                                .variant(
                                                    if entry.name == gallery.options.component {
                                                        ui::Variant::Secondary
                                                    } else {
                                                        ui::Variant::Ghost
                                                    },
                                                )
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
                                    let padding = if compact { 12. } else { 32. };
                                    ui::surface(t.background, None, 0., padding, || {
                                        let content_width = (width - padding * 2.).max(1.);
                                        if let Some(name) = gallery.options.showcase.clone() {
                                            showcases::render(
                                                &name,
                                                &mut gallery.showcase_state,
                                                content_width,
                                            );
                                        } else {
                                            ui::width(content_width.min(1040.), || {
                                                component(
                                                    gallery,
                                                    content_width.min(1040.),
                                                    navigation,
                                                )
                                            });
                                        }
                                    });
                                });
                            });
                        },
                    );
                });
        },
        (),
    );
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
            let items = component_items(gallery);
            gallery.component_picker.selected = gallery
                .catalogue
                .components
                .iter()
                .position(|e| e.name == component);
            ui::Select::new(&items)
                .width(width)
                .show(&mut gallery.component_picker);
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
                                horizontal_scroll(|| {
                                    // The examples retain their reference layout; narrow screens can
                                    // pan wide compositions without shrinking text or hit targets.
                                    ui::surface(t.background, None, 10., 20., || {
                                        ui::min_size(
                                            ui::Vec2::new((width - 42.).max(0.), 144.),
                                            || examples::render(&component, variant, state, false),
                                        );
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
    offset: std::cell::Cell<f32>,
    overflow: std::cell::Cell<f32>,
    height: std::cell::Cell<f32>,
    focus: std::cell::Cell<Option<yakui::WidgetId>>,
}
impl yakui::widget::Widget for PageScroll {
    type Props<'a> = &'a str;
    type Response = ();
    fn new() -> Self {
        Self {
            route: String::new(),
            offset: 0.0.into(),
            overflow: 0.0.into(),
            height: 0.0.into(),
            focus: None.into(),
        }
    }
    fn update(&mut self, route: &str) {
        if self.route != route {
            self.route = route.to_owned();
            self.offset.set(0.);
            self.focus.set(None);
        }
    }
    fn layout(
        &self,
        mut ctx: yakui::widget::LayoutContext<'_>,
        constraints: yakui::Constraints,
    ) -> ui::Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let mut content = ui::Vec2::ZERO;
        for &child in &ctx.dom.get_current().children {
            content = content.max(ctx.calculate_layout(
                child,
                yakui::Constraints {
                    min: ui::Vec2::ZERO,
                    max: ui::Vec2::new(constraints.max.x, f32::INFINITY),
                },
            ));
        }
        let size = constraints.constrain(content);
        self.overflow.set((content.y - size.y).max(0.));
        let focus = ctx.input.focus();
        let focus_changed = self.focus.replace(focus) != focus;
        let resized = self.height.replace(size.y) != size.y;
        if focus_changed || resized {
            if let Some(rect) = focus.and_then(|id| ctx.rect_relative_to(id, ctx.dom.current())) {
                // The software keyboard changes the viewport after focus has already
                // moved. Reveal the field again on resize, without resetting scroll.
                let offset = self.offset.get();
                if rect.max().y - offset > size.y {
                    self.offset.set(rect.max().y - size.y);
                }
                if rect.pos().y < self.offset.get() {
                    self.offset.set(rect.pos().y);
                }
            }
        }
        self.offset
            .set(self.offset.get().clamp(0., self.overflow.get()));
        for &child in &ctx.dom.get_current().children {
            ctx.layout
                .set_pos(child, ui::Vec2::new(0., -self.offset.get()));
        }
        size
    }
    fn event_interest(&self) -> yakui::event::EventInterest {
        yakui::event::EventInterest::MOUSE_INSIDE
    }
    fn event(
        &mut self,
        _: yakui::widget::EventContext<'_>,
        event: &yakui::event::WidgetEvent,
    ) -> yakui::event::EventResponse {
        if let yakui::event::WidgetEvent::MouseScroll { delta, .. } = event {
            let next = (self.offset.get() + delta.y).clamp(0., self.overflow.get());
            if next != self.offset.get() {
                self.offset.set(next);
                return yakui::event::EventResponse::Sink;
            }
        }
        yakui::event::EventResponse::Bubble
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

fn component_items(gallery: &Gallery) -> Vec<ui::SelectItem> {
    gallery
        .catalogue
        .components
        .iter()
        .enumerate()
        .map(|(i, entry)| ui::SelectItem::new(i, &entry.title))
        .collect()
}
pub fn overlays(gallery: &mut Gallery) {
    let items = component_items(gallery);
    if ui::SelectContent::new(&items).show(&mut gallery.component_picker) {
        if let Some(index) = gallery.component_picker.selected {
            gallery.options.component = gallery.catalogue.components[index].name.clone();
            gallery.options.variant = "default".into();
        }
    }
}

/// A horizontal viewport for reference previews and tables. Vertical gestures
/// bubble to the page; horizontal gestures stay here only while there is overflow.
#[derive(Debug)]
struct HorizontalScroll {
    offset: std::cell::Cell<f32>,
    overflow: std::cell::Cell<f32>,
}
impl yakui::widget::Widget for HorizontalScroll {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self {
            offset: 0.0.into(),
            overflow: 0.0.into(),
        }
    }
    fn update(&mut self, _: ()) {}
    fn layout(
        &self,
        mut ctx: yakui::widget::LayoutContext<'_>,
        constraints: yakui::Constraints,
    ) -> ui::Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let mut content = ui::Vec2::ZERO;
        for &child in &ctx.dom.get_current().children {
            content = content.max(ctx.calculate_layout(
                child,
                yakui::Constraints {
                    min: ui::Vec2::ZERO,
                    max: ui::Vec2::new(f32::INFINITY, constraints.max.y),
                },
            ));
        }
        let size = constraints.constrain(content);
        self.overflow.set((content.x - size.x).max(0.));
        self.offset
            .set(self.offset.get().clamp(0., self.overflow.get()));
        for &child in &ctx.dom.get_current().children {
            ctx.layout
                .set_pos(child, ui::Vec2::new(-self.offset.get(), 0.));
        }
        size
    }
    fn event_interest(&self) -> yakui::event::EventInterest {
        yakui::event::EventInterest::MOUSE_INSIDE
    }
    fn event(
        &mut self,
        _: yakui::widget::EventContext<'_>,
        event: &yakui::event::WidgetEvent,
    ) -> yakui::event::EventResponse {
        if let yakui::event::WidgetEvent::MouseScroll { delta, .. } = event {
            if delta.x.abs() > delta.y.abs() && self.overflow.get() > 0. {
                self.offset
                    .set((self.offset.get() + delta.x).clamp(0., self.overflow.get()));
                return yakui::event::EventResponse::Sink;
            }
        }
        yakui::event::EventResponse::Bubble
    }
}
pub(crate) fn horizontal_scroll(children: impl FnOnce()) {
    yakui::util::widget_children::<HorizontalScroll, _>(children, ());
}

/// Measure the header first so wrapping and text metrics determine the space
/// left for the page. The body receives that remaining height as a constraint.
#[derive(Debug)]
struct GalleryFrame;
impl yakui::widget::Widget for GalleryFrame {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(
        &self,
        mut ctx: yakui::widget::LayoutContext<'_>,
        constraints: yakui::Constraints,
    ) -> ui::Vec2 {
        let mut top = 0.;
        for &child in &ctx.dom.get_current().children {
            let size = ctx.calculate_layout(
                child,
                yakui::Constraints {
                    min: ui::Vec2::ZERO,
                    max: ui::Vec2::new(constraints.max.x, (constraints.max.y - top).max(0.)),
                },
            );
            ctx.layout.set_pos(child, ui::Vec2::new(0., top));
            top += size.y;
        }
        constraints.constrain(ui::Vec2::new(constraints.max.x, top))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yakui::{
        Rect, Vec2,
        event::Event,
        input::{KeyCode, MouseButton},
    };

    fn frame(yak: &mut yakui::Yakui, gallery: &mut Gallery, size: Vec2) {
        gallery.viewport = size;
        yak.set_surface_size(size);
        yak.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
        yak.start();
        ui::animation::with_motion_enabled(false, || gallery.render());
        yak.finish();
    }
    fn widgets(yak: &yakui::Yakui, suffix: &str) -> Vec<yakui::WidgetId> {
        fn visit(
            yak: &yakui::Yakui,
            id: yakui::WidgetId,
            suffix: &str,
            out: &mut Vec<yakui::WidgetId>,
        ) {
            let node = yak.dom().get(id).unwrap();
            if node.widget.type_name().ends_with(suffix) {
                out.push(id);
            }
            for &child in &node.children {
                visit(yak, child, suffix, out);
            }
        }
        let mut result = Vec::new();
        visit(yak, yak.dom().root(), suffix, &mut result);
        result
    }
    fn key(yak: &mut yakui::Yakui, key: KeyCode) {
        for down in [true, false] {
            yak.handle_event(Event::KeyChanged {
                key,
                down,
                modifiers: None,
            });
        }
    }
    #[test]
    fn catalogue_and_showcases_keep_the_page_inside_the_viewport() {
        let mut yak = crate::new_yak();
        let mut gallery = Gallery::new(crate::Options::default());
        for width in [320., 390., 768., 1200.] {
            for showcase in [None, Some("dashboard"), Some("settings"), Some("checkout")] {
                gallery.options.showcase = showcase.map(str::to_owned);
                frame(&mut yak, &mut gallery, Vec2::new(width, 700.));
                let id = widgets(&yak, "::PageScroll")[0];
                let rect = yak.layout_dom().get(id).unwrap().rect;
                assert!(rect.max().y <= 700.1, "{width} {showcase:?}: {rect:?}");
                assert!(
                    rect.max().x <= width + 0.1,
                    "{width} {showcase:?}: {rect:?}"
                );
                assert!(rect.size().y > 400.);
            }
        }
    }
    #[test]
    fn narrow_catalogue_previews_have_finite_layouts() {
        let mut yak = crate::new_yak();
        let mut gallery = Gallery::new(crate::Options::default());
        for component in examples::AVAILABLE {
            gallery.options.component = (*component).into();
            frame(&mut yak, &mut gallery, Vec2::new(390., 700.));
            for id in widgets(&yak, "") {
                let rect = yak.layout_dom().get(id).unwrap().rect;
                assert!(
                    rect.pos().is_finite() && rect.size().is_finite(),
                    "{component}: {rect:?}"
                );
            }
        }
    }
    #[test]
    fn compact_picker_can_reach_the_last_catalogue_entry() {
        let mut yak = crate::new_yak();
        let mut gallery = Gallery::new(crate::Options::default());
        let size = Vec2::new(390., 700.);
        frame(&mut yak, &mut gallery, size);
        // Select uses a real trigger and retained overlay focus, then commits via Enter.
        let trigger = widgets(&yak, "::ControlWidget")
            .into_iter()
            .find(|&id| {
                let rect = yak.layout_dom().get(id).unwrap().rect;
                rect.size().x > 300. && rect.pos().y < 350.
            })
            .expect("component picker trigger");
        yak.request_focus(Some(trigger));
        key(&mut yak, KeyCode::ArrowDown);
        frame(&mut yak, &mut gallery, size);
        frame(&mut yak, &mut gallery, size);
        assert!(gallery.component_picker.popup.open);
        key(&mut yak, KeyCode::End);
        frame(&mut yak, &mut gallery, size);
        key(&mut yak, KeyCode::Enter);
        frame(&mut yak, &mut gallery, size);
        assert_eq!(
            gallery.options.component,
            gallery.catalogue.components.last().unwrap().name
        );
    }
    #[test]
    fn swiping_a_preview_scrolls_the_page_without_clicking_the_button() {
        let mut yak = crate::new_yak();
        let mut gallery = Gallery::new(crate::Options::default());
        let size = Vec2::new(390., 700.);
        frame(&mut yak, &mut gallery, size);
        let preview = widgets(&yak, "::HorizontalScroll")[1];
        let rect = yak.layout_dom().get(preview).unwrap().rect;
        let start = rect.pos() + Vec2::new(35., 35.);
        yak.handle_event(Event::CursorMoved(Some(start)));
        yak.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down: true,
        });
        yak.handle_event(Event::CursorMoved(None));
        yak.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down: false,
        });
        yak.handle_event(Event::CursorMoved(Some(start)));
        yak.handle_event(Event::MouseScroll {
            delta: Vec2::new(0., 100.),
        });
        frame(&mut yak, &mut gallery, size);
        assert!(yak.layout_dom().get(preview).unwrap().rect.pos().y < rect.pos().y - 90.);
        assert!(gallery.states.values().all(|s| s.count == 0));
        gallery.options.component = "sidebar".into();
        frame(&mut yak, &mut gallery, size);
        let preview = widgets(&yak, "::HorizontalScroll")[1];
        let start = yak.layout_dom().get(preview).unwrap().rect.pos() + Vec2::new(10., 10.);
        let child = yak.dom().get(preview).unwrap().children[0];
        let before = yak.layout_dom().get(child).unwrap().rect.pos().x;
        yak.handle_event(Event::CursorMoved(Some(start)));
        yak.handle_event(Event::MouseScroll {
            delta: Vec2::new(100., 0.),
        });
        frame(&mut yak, &mut gallery, size);
        assert!(yak.layout_dom().get(child).unwrap().rect.pos().x < before - 90.);
    }
    #[test]
    fn keyboard_resize_keeps_the_focused_checkout_field_visible() {
        let mut yak = crate::new_yak();
        let mut gallery = Gallery::new(crate::Options {
            showcase: Some("checkout".into()),
            ..Default::default()
        });
        frame(&mut yak, &mut gallery, Vec2::new(390., 800.));
        let editor = widgets(&yak, "::Editor")[2];
        yak.request_focus(Some(editor));
        frame(&mut yak, &mut gallery, Vec2::new(390., 800.));
        frame(&mut yak, &mut gallery, Vec2::new(390., 400.));
        let rect = yak.layout_dom().get(editor).unwrap().rect;
        let page = yak
            .layout_dom()
            .get(widgets(&yak, "::PageScroll")[0])
            .unwrap()
            .rect;
        assert!(
            rect.pos().y >= page.pos().y && rect.max().y <= page.max().y + 0.1,
            "{rect:?} {page:?}"
        );
        assert!(yak.text_input_enabled());
    }
}
