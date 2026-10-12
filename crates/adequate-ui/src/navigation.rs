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
    radio: bool,
    line: bool,
}
#[derive(Debug)]
struct ChoiceWidget {
    props: ChoiceProps,
    control: ControlWidget,
    context: Option<Rc<ChoiceContext>>,
    line_color: Color,
}
impl Widget for ChoiceWidget {
    type Props<'a> = ChoiceProps;
    type Response = ControlResponse;
    fn new() -> Self {
        Self {
            props: ChoiceProps {
                index: 0,
                selected: 0,
                radio: false,
                line: false,
            },
            control: ControlWidget::new(),
            context: None,
            line_color: Color::CLEAR,
        }
    }
    fn update(&mut self, props: ChoiceProps) -> ControlResponse {
        self.props = props;
        self.context = yakui::util::read_scope::<ChoiceContext>();
        let t = theme();
        self.line_color = t.foreground;
        self.control.update(Control {
            disabled: false,
            radius: if props.radio { 999. } else { t.radius },
            ring: t.ring,
            focus_size: if props.radio {
                Some(crate::Vec2::splat(16.))
            } else {
                None
            },
        })
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        self.control.paint(PaintContext {
            dom: ctx.dom,
            layout: ctx.layout,
            paint: ctx.paint,
        });
        if self.props.line && self.props.index == self.props.selected {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            crate::icons::line(
                &mut ctx,
                crate::Vec2::new(rect.pos().x, rect.max().y + 3.),
                crate::Vec2::new(rect.max().x, rect.max().y + 3.),
                2.,
                self.line_color,
            );
        }
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
/// Build a single tab stop with selection following arrow/Home/End navigation.
/// The caller owns selection; Yakui retains only the event awaiting the next frame.
#[track_caller]
pub(crate) fn choice_group(
    count: usize,
    selected: &mut usize,
    mut item: impl FnMut(usize, &mut usize, ControlResponse),
) {
    *selected = (*selected).min(count.saturating_sub(1));
    let ids = Rc::new(RefCell::new(Vec::new()));
    crate::layout::with_state(
        || Rc::new(Cell::new(None::<usize>)),
        |pending| {
            if let Some(next) = pending.borrow().take() {
                *selected = next.min(count.saturating_sub(1));
            }
            Scope::new(ChoiceContext {
                ids: ids.clone(),
                pending: pending.borrow().clone(),
            })
            .show(|| {
                column(8., || {
                    for index in 0..count {
                        let dom = context::dom();
                        let response = dom.begin_widget::<ChoiceWidget>(ChoiceProps {
                            index,
                            selected: *selected,
                            radio: true,
                            line: false,
                        });
                        let id = response.id;
                        ids.borrow_mut().push(id);
                        let response = response.into_inner();
                        if response.clicked {
                            *selected = index;
                        }
                        item(index, selected, response);
                        dom.end_widget::<ChoiceWidget>(id);
                    }
                });
            });
        },
    )
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
    crate::layout::with_state(
        || Rc::new(Cell::new(None::<usize>)),
        |persisted| {
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
                                        radio: false,
                                        line: matches!(variant, TabsVariant::Line),
                                    });
                                    let id = response.id;
                                    ids.borrow_mut().push(id);
                                    let res = response.into_inner();
                                    if res.clicked {
                                        *selected = index;
                                    }
                                    let fill = if *selected == index
                                        && matches!(variant, TabsVariant::Default)
                                    {
                                        t.background
                                    } else {
                                        Color::CLEAR
                                    };
                                    surface(fill, None, t.radius - 2., 0., || {
                                        yakui::pad(yakui::widgets::Pad::balanced(7., 3.), || {
                                            yakui::widgets::RenderText::with_style(
                                                (*option).to_owned(),
                                                t.text_style(
                                                    14.,
                                                    if *selected == index {
                                                        t.foreground
                                                    } else {
                                                        t.muted_foreground
                                                    },
                                                    500,
                                                ),
                                            )
                                            .show();
                                        });
                                    });
                                    dom.end_widget::<ChoiceWidget>(id);
                                }
                            })
                        },
                    )
                });
                content(*selected);
            });
        },
    )
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
