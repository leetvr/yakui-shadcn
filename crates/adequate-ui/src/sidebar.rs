//! A native collapsible sidebar using scoped sidebar tokens and ordinary components.
use crate::{Button, ButtonSize, Icon, Variant, Vec2};
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
            let collapsed = sidebar_width < 100.;
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
                                    if Button::new(if collapsed { "" } else { title })
                                        .icon(Icon::Menu)
                                        .variant(Variant::Ghost)
                                        .show()
                                        .clicked
                                    {
                                        state.collapsed = !state.collapsed;
                                        crate::animation::request_frame();
                                    }
                                    crate::scroll_area((size.y - 64.).max(0.), || {
                                        crate::column(4., || {
                                            for (i, link) in links.iter().enumerate() {
                                                let button = Button::new(if collapsed {
                                                    String::new()
                                                } else {
                                                    link.label.clone()
                                                })
                                                .icon(link.icon)
                                                .variant(if state.active == i {
                                                    Variant::Secondary
                                                } else {
                                                    Variant::Ghost
                                                });
                                                let response = if collapsed {
                                                    button.size(ButtonSize::Icon).show()
                                                } else {
                                                    button.show()
                                                };
                                                if response.clicked {
                                                    state.active = i;
                                                }
                                            }
                                        })
                                    });
                                })
                            });
                        })
                    });
                    crate::width((size.x - sidebar_width).max(0.), || {
                        crate::surface(t.background, None, 0., 24., || content(state.active))
                    });
                })
            });
        },
    );
}
