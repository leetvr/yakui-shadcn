//! Roving tab stops and selection are shared by tabs, radio groups and toolbars.
use crate::interaction::{Control, ControlWidget};
use crate::{Button, ControlResponse, Variant, column, row, surface, theme};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use yakui::{
    Color, WidgetId, context,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::KeyCode,
    widget::{EventContext, FocusPolicy, PaintContext, Widget},
    widgets::Scope,
};
#[derive(Debug, Clone)]
struct ChoiceContext {
    ids: Rc<RefCell<Vec<WidgetId>>>,
    pending: Rc<Cell<Option<usize>>>,
}
#[derive(Debug, Clone, Copy)]
struct ChoiceProps {
    index: usize,
    selected: usize,
}
#[derive(Debug)]
struct ChoiceWidget {
    props: ChoiceProps,
    control: ControlWidget,
    context: Option<Rc<ChoiceContext>>,
}
impl Widget for ChoiceWidget {
    type Props<'a> = ChoiceProps;
    type Response = ControlResponse;
    fn new() -> Self {
        Self {
            props: ChoiceProps {
                index: 0,
                selected: 0,
            },
            control: ControlWidget::new(),
            context: None,
        }
    }
    fn update(&mut self, props: ChoiceProps) -> ControlResponse {
        self.props = props;
        self.context = yakui::util::read_scope::<ChoiceContext>();
        let t = theme();
        self.control.update(Control {
            disabled: false,
            radius: t.radius,
            ring: t.ring,
        })
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        self.control.paint(ctx)
    }
    fn event_interest(&self) -> EventInterest {
        self.control.event_interest()
    }
    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::POINTER
            | FocusPolicy::DIRECTIONAL
            | if self.props.index == self.props.selected {
                FocusPolicy::SEQUENTIAL
            } else {
                FocusPolicy::empty()
            }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::KeyChanged {
            key, down: true, ..
        } = event
        {
            if let Some(group) = &self.context {
                let ids = group.ids.borrow();
                let count = ids.len();
                if count > 0 {
                    let next = match key {
                        KeyCode::ArrowRight | KeyCode::ArrowDown => {
                            Some((self.props.index + 1) % count)
                        }
                        KeyCode::ArrowLeft | KeyCode::ArrowUp => {
                            Some((self.props.index + count - 1) % count)
                        }
                        KeyCode::Home => Some(0),
                        KeyCode::End => Some(count - 1),
                        _ => None,
                    };
                    if let Some(next) = next {
                        group.pending.set(Some(next));
                        ctx.input.set_focus(Some(ids[next]), true);
                        return EventResponse::Sink;
                    }
                }
            }
        }
        self.control.event(ctx, event)
    }
}
/// Tab-list appearance from the documented default/line variants.
#[derive(Debug, Clone, Copy, Default)]
pub enum TabsVariant {
    #[default]
    Default,
    Line,
}
/// One tab stop with arrow/Home/End navigation and immediate selection.
#[track_caller]
pub fn tabs(
    options: &[&str],
    selected: &mut usize,
    variant: TabsVariant,
    content: impl FnOnce(usize),
) {
    let t = theme();
    let ids = Rc::new(RefCell::new(Vec::new()));
    // The scope widget owns the pending selection across frames. Caller state is
    // only changed here, so consumers never need a second selection store.
    let persisted = yakui::use_state(|| Rc::new(Cell::new(None::<usize>)));
    if let Some(next) = persisted.borrow().take() {
        *selected = next.min(options.len().saturating_sub(1));
    }
    let context = ChoiceContext {
        ids: ids.clone(),
        pending: persisted.borrow().clone(),
    };
    column(16., || {
        Scope::new(context).show(|| {
            surface(
                if matches!(variant, TabsVariant::Default) {
                    t.muted
                } else {
                    Color::CLEAR
                },
                None,
                t.radius,
                3.,
                || {
                    row(2., || {
                        for (index, option) in options.iter().enumerate() {
                            let dom = context::dom();
                            let response = dom.begin_widget::<ChoiceWidget>(ChoiceProps {
                                index,
                                selected: *selected,
                            });
                            let id = response.id;
                            ids.borrow_mut().push(id);
                            let res = response.into_inner();
                            if res.clicked {
                                *selected = index;
                            }
                            let fill =
                                if *selected == index && matches!(variant, TabsVariant::Default) {
                                    t.background
                                } else {
                                    Color::CLEAR
                                };
                            surface(fill, None, t.radius - 2., 5., || {
                                crate::label((*option).to_owned())
                            });
                            dom.end_widget::<ChoiceWidget>(id);
                        }
                    })
                },
            )
        });
        content(*selected);
    });
}
/// Expandable content with application-owned visibility.
#[track_caller]
pub fn collapsible(title: impl Into<String>, open: &mut bool, children: impl FnOnce()) {
    column(8., || {
        if Button::new(title)
            .variant(Variant::Ghost)
            .icon(if *open {
                crate::Icon::ChevronDown
            } else {
                crate::Icon::ChevronRight
            })
            .show()
            .clicked
        {
            *open = !*open;
        }
        if *open {
            children();
        }
    });
}
/// One accordion item, including the dividing rule and expanded-content spacing.
#[track_caller]
pub fn accordion_item(
    title: impl Into<String>,
    open: &mut bool,
    width: f32,
    children: impl FnOnce(),
) {
    crate::width(width, || {
        column(0., || {
            if Button::new(title)
                .variant(Variant::Ghost)
                .icon(if *open {
                    crate::Icon::ChevronDown
                } else {
                    crate::Icon::ChevronRight
                })
                .show()
                .clicked
            {
                *open = !*open;
            }
            if *open {
                surface(Color::CLEAR, None, 0., 10., children);
            }
            crate::separator(width);
        })
    });
}
/// Breadcrumb links report an index so routing remains application-owned.
#[track_caller]
pub fn breadcrumb(items: &[&str]) -> Option<usize> {
    let mut clicked = None;
    row(6., || {
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                crate::icon_colored(crate::Icon::ChevronRight, 14., theme().muted_foreground);
            }
            if i == items.len() - 1 {
                crate::text((*item).to_owned());
            } else if Button::new((*item).to_owned())
                .variant(Variant::Link)
                .show()
                .clicked
            {
                clicked = Some(i);
            }
        }
    });
    clicked
}
/// Page selection with previous/next and numbered actions. Page indices start at zero.
#[track_caller]
pub fn pagination(page: &mut usize, pages: usize) {
    if pages == 0 {
        return;
    }
    *page = (*page).min(pages - 1);
    row(4., || {
        if Button::new("Previous")
            .variant(Variant::Ghost)
            .disabled(*page == 0)
            .show()
            .clicked
        {
            *page = page.saturating_sub(1);
        }
        for i in 0..pages {
            if Button::new((i + 1).to_string())
                .variant(if i == *page {
                    Variant::Outline
                } else {
                    Variant::Ghost
                })
                .show()
                .clicked
            {
                *page = i;
            }
        }
        if Button::new("Next")
            .variant(Variant::Ghost)
            .disabled(*page + 1 == pages)
            .show()
            .clicked
        {
            *page = (*page + 1).min(pages - 1);
        }
    });
}
