//! An explicit native toast queue; expiration and actions stay local to the application.
use crate::{Button, ButtonSize, Variant, Vec2, column, row, surface, theme};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    #[default]
    Default,
    Success,
    Error,
    Info,
    Warning,
    Loading,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub kind: ToastKind,
    pub action: Option<String>,
    pub duration: Option<Duration>,
    created: Instant,
}
#[derive(Debug, Default)]
pub struct ToastManager {
    pub toasts: Vec<Toast>,
    next_id: u64,
    visual: Vec<VisualToast>,
}
#[derive(Debug)]
struct VisualToast {
    slot: usize,
    toast: Toast,
    transition: crate::animation::Transition<f32>,
    visible: bool,
    progress: f32,
}
impl ToastManager {
    pub fn add(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        kind: ToastKind,
    ) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.toasts.push(Toast {
            id,
            title: title.into(),
            description: description.into(),
            kind,
            action: None,
            duration: Some(Duration::from_secs(5)),
            created: Instant::now(),
        });
        id
    }
    pub fn close(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }
    pub fn update(&mut self) {
        self.toasts.retain(|t| {
            t.duration
                .is_none_or(|duration| t.created.elapsed() < duration)
        });
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct ToastResponse {
    pub action: Option<u64>,
    pub dismissed: Option<u64>,
}
#[derive(Debug, Clone, Copy)]
struct ToastLayout {
    id: u64,
    visible: bool,
    progress: f32,
}
#[derive(Debug)]
struct ToastViewport {
    rows: Vec<Option<ToastLayout>>,
    order: Vec<usize>,
    positions: RefCell<HashMap<u64, crate::animation::Transition<f32>>>,
    now: Duration,
    enabled: bool,
    clock: Option<Rc<RefCell<crate::animation::FrameClock>>>,
}
impl yakui::widget::Widget for ToastViewport {
    type Props<'a> = Vec<Option<ToastLayout>>;
    type Response = ();
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            order: Vec::new(),
            positions: RefCell::new(HashMap::new()),
            now: Duration::ZERO,
            enabled: true,
            clock: None,
        }
    }
    fn update(&mut self, rows: Self::Props<'_>) {
        self.rows = rows;
        self.order = (0..self.rows.len())
            .filter(|&slot| self.rows[slot].is_some())
            .collect();
        self.order.sort_by_key(|&slot| self.rows[slot].unwrap().id);
        self.now = crate::animation::now();
        self.enabled = crate::animation::enabled();
        self.clock = Some(crate::animation::frame_clock());
        self.positions
            .get_mut()
            .retain(|id, _| self.rows.iter().flatten().any(|row| row.id == *id));
    }
    fn layout(&self, mut ctx: yakui::widget::LayoutContext<'_>, input: yakui::Constraints) -> Vec2 {
        ctx.layout.new_layer(ctx.dom);
        let size = input.max;
        let width = (size.x - 32.).clamp(0., 356.);
        let mut bottom = 16.;
        let mut positions = self.positions.borrow_mut();
        for (&child, row) in ctx.dom.get_current().children.iter().zip(&self.rows) {
            if row.is_none() {
                ctx.calculate_layout(child, yakui::Constraints::tight(Vec2::ZERO));
            }
        }
        for &slot in self.order.iter().rev() {
            let child = ctx.dom.get_current().children[slot];
            let row = self.rows[slot].unwrap();
            let body = ctx.calculate_layout(
                child,
                yakui::Constraints {
                    min: Vec2::new(width, 0.),
                    max: Vec2::new(width, (size.y - 32.).max(0.)),
                },
            );
            let target = size.y - bottom - body.y;
            let position = positions
                .entry(row.id)
                .or_insert_with(|| crate::animation::Transition::new(target));
            // Closing rows retain their old destination while their opacity exits.
            // Live rows immediately reclaim that space, from their current sample.
            // Key by application ID: removing a middle toast must not transplant
            // its layout transition onto the next retained row.
            if row.visible {
                if self.enabled {
                    position.set_target(target, toast_motion(), self.now);
                } else {
                    position.snap(target);
                }
                bottom += body.y + 8.;
            }
            if position.is_active(self.now) {
                self.clock.as_ref().unwrap().borrow_mut().active = true;
            }
            ctx.layout.set_pos(
                child,
                Vec2::new(
                    (size.x - width - 16.).max(0.),
                    position.value(self.now) + 16. * (1. - row.progress),
                ),
            );
            if !row.visible {
                ctx.layout.disable_input(ctx.dom, child);
            }
        }
        size
    }
    fn paint(&self, mut ctx: yakui::widget::PaintContext<'_>) {
        // Slot reuse must not change overlap order: newest toasts paint on top,
        // matching the reverse layout order used for pointer registration.
        for &slot in &self.order {
            ctx.paint(ctx.dom.get_current().children[slot]);
        }
    }
    fn navigate(
        &self,
        ctx: yakui::widget::NavigateContext<'_>,
        direction: yakui_core::navigation::NavDirection,
    ) -> Option<yakui::WidgetId> {
        let visit = |&slot: &usize| {
            self.rows[slot]
                .unwrap()
                .visible
                .then(|| ctx.try_navigate(ctx.dom.get_current().children[slot], direction))
                .flatten()
        };
        if matches!(direction, yakui_core::navigation::NavDirection::Previous) {
            self.order.iter().rev().find_map(visit)
        } else {
            self.order.iter().find_map(visit)
        }
    }
}
fn toast_motion() -> crate::animation::Motion {
    crate::animation::Motion::new(
        Duration::from_millis(200),
        crate::animation::Easing::EaseOut,
    )
}
/// Render at the application root, after content and dialogs. Actions return toast IDs.
#[track_caller]
pub fn toast_viewport(manager: &mut ToastManager) -> ToastResponse {
    manager.update();
    let now = crate::animation::now();
    let timing = toast_motion();
    for toast in &manager.toasts {
        if let Some(visual) = manager.visual.iter_mut().find(|v| v.toast.id == toast.id) {
            if visual.toast != *toast {
                visual.toast.clone_from(toast);
            }
        } else {
            let slot = (0..)
                .find(|slot| manager.visual.iter().all(|v| v.slot != *slot))
                .unwrap();
            manager.visual.push(VisualToast {
                slot,
                toast: toast.clone(),
                transition: crate::animation::Transition::new(0.),
                visible: true,
                progress: 0.,
            });
        }
    }
    for visual in &mut manager.visual {
        visual.visible = manager
            .toasts
            .iter()
            .any(|toast| toast.id == visual.toast.id);
        let target = if visual.visible { 1. } else { 0. };
        if crate::animation::enabled() {
            visual.transition.set_target(target, timing, now);
        } else {
            visual.transition.snap(target);
        }
        visual.progress = visual.transition.value(now);
        if visual.transition.is_active(now) {
            crate::animation::request_frame();
        }
    }
    // Application state removes/expires toasts immediately; paint retains the
    // snapshot until exit ends. Transition state is keyed by toast ID, so row
    // removal cannot transfer another toast's animation progress to this one.
    manager.visual.retain(|visual| {
        visual.visible || visual.transition.is_active(now) || visual.progress > 0.
    });
    // Yakui keys widgets by tree position. Leave empty slots while other rows
    // survive, so removing a toast cannot transfer pressed/focused button state
    // to its neighbour. Freed slots are reused and trailing slots are trimmed.
    let slots = manager.visual.iter().map(|v| v.slot + 1).max().unwrap_or(0);
    let rows: Vec<_> = (0..slots)
        .map(|slot| manager.visual.iter().find(|v| v.slot == slot))
        .collect();
    let t = theme();
    let mut response = ToastResponse::default();
    yakui::util::widget_children::<ToastViewport, _>(
        || {
            for visual in &rows {
                yakui::widgets::ConstrainedBox::new(yakui::Constraints::none()).show(|| {
                    let Some(visual) = visual else {
                        return;
                    };
                    let toast = &visual.toast;
                    crate::animation::motion(
                        crate::animation::MotionStyle {
                            opacity: visual.progress,
                            ..Default::default()
                        },
                        || {
                            surface(t.popover, Some(t.border), t.radius, 12., || {
                                row(12., || {
                                    if matches!(toast.kind, ToastKind::Loading) {
                                        crate::spinner_auto(16.);
                                    } else if matches!(toast.kind, ToastKind::Success) {
                                        crate::icon(crate::Icon::Check, 16.);
                                    }
                                    column(4., || {
                                        crate::label(&toast.title);
                                        crate::muted(&toast.description);
                                    });
                                    if let Some(action) = &toast.action {
                                        if Button::new(action)
                                            .size(ButtonSize::Sm)
                                            .disabled(!visual.visible)
                                            .show()
                                            .clicked
                                        {
                                            response.action = Some(toast.id);
                                        }
                                    }
                                    if Button::new("")
                                        .variant(Variant::Ghost)
                                        .size(ButtonSize::IconXs)
                                        .icon(crate::Icon::X)
                                        .disabled(!visual.visible)
                                        .show()
                                        .clicked
                                    {
                                        response.dismissed = Some(toast.id);
                                    }
                                })
                            });
                        },
                    );
                });
            }
        },
        rows.iter()
            .map(|visual| {
                visual.map(|visual| ToastLayout {
                    id: visual.toast.id,
                    visible: visual.visible,
                    progress: visual.progress,
                })
            })
            .collect(),
    );
    if let Some(id) = response.dismissed {
        manager.close(id);
    }
    response
}
