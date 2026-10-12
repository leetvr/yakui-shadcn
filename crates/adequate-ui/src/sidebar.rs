use crate::{ControlResponse, Icon, Vec2};
use yakui::{
    Constraints,
    widget::{LayoutContext, Widget},
};
#[derive(Debug, Clone)]
pub struct SidebarLink {
    pub label: String,
    pub icon: Icon,
}
impl SidebarLink {
    pub fn new(label: impl Into<String>, icon: Icon) -> Self {
        Self {
            label: label.into(),
            icon,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct SidebarState {
    pub collapsed: bool,
    pub active: usize,
}
#[derive(Debug)]
struct Label;
impl Widget for Label {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        // Measure at natural width and clip to the shrinking sidebar. Keeping
        // the text declared prevents wrapping or replacing it halfway through
        // collapse, and leaves the icon and control identity stationary.
        let mut natural = Vec2::ZERO;
        for &child in &ctx.dom.get_current().children {
            natural = natural.max(
                ctx.calculate_layout(child, Constraints::loose(Vec2::new(f32::INFINITY, 20.))),
            );
        }
        constraints.constrain(natural)
    }
}
#[track_caller]
fn sidebar_button(label: &str, icon: Icon, width: f32, selected: bool) -> ControlResponse {
    let t = crate::theme();
    crate::interaction::control(
        crate::interaction::Control {
            disabled: false,
            radius: t.radius,
            ring: t.ring,
            focus_size: None,
        },
        |response| {
            let fill = if selected || response.hovering || response.pressed {
                t.accent
            } else {
                t.background
            };
            crate::animation::animate(fill, Default::default(), |fill| {
                crate::size(Vec2::new(width, 32.), || {
                    crate::surface(fill, None, t.radius, 8., || {
                        crate::row(0., || {
                            crate::icon(icon, 16.);
                            crate::width((width - 32.).max(0.), || {
                                yakui::util::widget_children::<Label, _>(
                                    || {
                                        yakui::pad(
                                            yakui::widgets::Pad {
                                                left: 8.,
                                                ..Default::default()
                                            },
                                            || {
                                                crate::label(label);
                                            },
                                        );
                                    },
                                    (),
                                );
                            });
                        });
                    });
                });
            });
        },
    )
}
/// Sidebar/content composition. The caller receives the active route index and
/// builds its content with public components. No hidden renderer/window state.
#[track_caller]
pub fn sidebar(
    size: Vec2,
    state: &mut SidebarState,
    title: &str,
    links: &[SidebarLink],
    content: impl FnOnce(usize),
) {
    let t = crate::theme();
    let sidebar_width = if state.collapsed { 48. } else { 240. };
    crate::animation::animate(
        sidebar_width,
        crate::animation::Motion::new(
            std::time::Duration::from_millis(200),
            crate::animation::Easing::Linear,
        ),
        |sidebar_width| {
            crate::size(size, || {
                crate::row(0., || {
                    crate::width(sidebar_width, || {
                        crate::surface(t.sidebar, Some(t.sidebar_border), 0., 8., || {
                            let mut inner = (*t).clone();
                            inner.background = t.sidebar;
                            inner.foreground = t.sidebar_foreground;
                            inner.primary = t.sidebar_primary;
                            inner.primary_foreground = t.sidebar_primary_foreground;
                            inner.accent = t.sidebar_accent;
                            inner.secondary = t.sidebar_accent;
                            inner.ring = t.sidebar_ring;
                            crate::with_theme(inner, || {
                                crate::column(16., || {
                                    if sidebar_button(title, Icon::Menu, sidebar_width - 16., false)
                                        .clicked
                                    {
                                        state.collapsed = !state.collapsed;
                                    }
                                    crate::scroll_area((size.y - 64.).max(0.), || {
                                        crate::column(4., || {
                                            for (i, link) in links.iter().enumerate() {
                                                if sidebar_button(
                                                    &link.label,
                                                    link.icon,
                                                    sidebar_width - 16.,
                                                    state.active == i,
                                                )
                                                .clicked
                                                {
                                                    state.active = i;
                                                }
                                            }
                                        });
                                    });
                                });
                            });
                        });
                    });
                    crate::width((size.x - sidebar_width).max(0.), || {
                        crate::surface(t.background, None, 0., 24., || content(state.active));
                    });
                });
            });
        },
    );
}
