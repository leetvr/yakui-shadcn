//! Native menu entries, keyboard traversal, typeahead and caller-owned checked state.
use crate::interaction::{Control, ControlWidget};
use crate::{Color, ControlResponse, OverlayState, theme};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};
use yakui::{
    WidgetId, context,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::KeyCode,
    widget::{EventContext, FocusPolicy, PaintContext, Widget},
    widgets::Scope,
};
/// Stateful menu item kinds; nested items are ordinary owned Rust values.
#[derive(Debug, Clone)]
pub enum MenuKind {
    Action,
    Checkbox(bool),
    Radio { group: u64, selected: bool },
    Submenu(Vec<MenuItem>),
    Separator,
    Label,
}
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: u64,
    pub label: String,
    pub shortcut: Option<String>,
    pub disabled: bool,
    pub destructive: bool,
    pub kind: MenuKind,
}
impl MenuItem {
    pub fn action(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            disabled: false,
            destructive: false,
            kind: MenuKind::Action,
        }
    }
    pub fn checkbox(id: u64, label: impl Into<String>, checked: bool) -> Self {
        Self {
            kind: MenuKind::Checkbox(checked),
            ..Self::action(id, label)
        }
    }
    pub fn separator() -> Self {
        Self {
            kind: MenuKind::Separator,
            ..Self::action(0, "")
        }
    }
    pub fn submenu(id: u64, label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            kind: MenuKind::Submenu(items),
            ..Self::action(id, label)
        }
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}
/// Application-owned menu visibility and submenu path.
#[derive(Debug, Default, Clone)]
pub struct MenuState {
    pub popup: OverlayState,
    path: Vec<usize>,
    submenus: Vec<OverlayState>,
}
impl MenuState {
    pub fn open_from(&mut self, response: ControlResponse) {
        self.path.clear();
        self.submenus.clear();
        self.popup.open_from(response);
    }
    pub fn close(&mut self) {
        self.popup.close();
        self.path.clear();
        self.submenus.clear();
    }
}
#[derive(Debug, Clone, Copy)]
enum MenuCommand {
    Activate(usize),
    Open(usize),
}
#[derive(Debug, Clone)]
pub(crate) struct MenuContext {
    ids: Rc<RefCell<Vec<(usize, WidgetId, String)>>>,
    pending: Rc<Cell<Option<MenuCommand>>>,
    search: Rc<RefCell<(String, Instant)>>,
}
impl MenuContext {
    pub(crate) fn editor_key(&self, ctx: &EventContext<'_>, event: &WidgetEvent) -> bool {
        if let WidgetEvent::KeyChanged {
            key, down: true, ..
        } = event
        {
            let ids = self.ids.borrow();
            let target = match key {
                KeyCode::ArrowDown | KeyCode::Enter | KeyCode::NumpadEnter => ids.first(),
                KeyCode::ArrowUp => ids.last(),
                _ => None,
            };
            if let Some((index, id, _)) = target {
                if matches!(key, KeyCode::Enter | KeyCode::NumpadEnter) {
                    self.pending.set(Some(MenuCommand::Activate(*index)));
                } else {
                    ctx.input.set_focus(Some(*id), true);
                }
                return true;
            }
        }
        false
    }
}
#[derive(Debug, Clone, Copy)]
struct EntryProps {
    index: usize,
    disabled: bool,
    submenu: bool,
}
#[derive(Debug)]
struct EntryWidget {
    props: EntryProps,
    control: ControlWidget,
    context: Option<Rc<MenuContext>>,
    dismiss: Option<Rc<crate::overlay::DismissContext>>,
}
impl Widget for EntryWidget {
    type Props<'a> = EntryProps;
    type Response = ControlResponse;
    fn new() -> Self {
        Self {
            props: EntryProps {
                index: 0,
                disabled: false,
                submenu: false,
            },
            control: ControlWidget::new(),
            context: None,
            dismiss: None,
        }
    }
    fn update(&mut self, props: EntryProps) -> ControlResponse {
        self.props = props;
        self.context = yakui::util::read_scope();
        self.dismiss = yakui::util::read_scope();
        let t = theme();
        self.control.update(Control {
            disabled: props.disabled,
            radius: t.radius * 0.6,
            ring: t.ring,
            focus_size: None,
        })
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        self.control.paint(ctx)
    }
    fn event_interest(&self) -> EventInterest {
        self.control.event_interest()
    }
    fn focus_policy(&self) -> FocusPolicy {
        self.control.focus_policy()
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if self.props.disabled {
            return EventResponse::Bubble;
        }
        if let Some(group) = &self.context {
            let ids = group.ids.borrow();
            let pos = ids
                .iter()
                .position(|(i, _, _)| *i == self.props.index)
                .unwrap_or(0);
            let count = ids.len();
            if let WidgetEvent::KeyChanged {
                key, down: true, ..
            } = event
            {
                if count > 0 {
                    let target = match key {
                        KeyCode::ArrowDown => Some((pos + 1) % count),
                        KeyCode::ArrowUp => Some((pos + count - 1) % count),
                        KeyCode::Home => Some(0),
                        KeyCode::End => Some(count - 1),
                        _ => None,
                    };
                    if let Some(target) = target {
                        ctx.input.set_focus(Some(ids[target].1), true);
                        return EventResponse::Sink;
                    }
                }
                if *key == KeyCode::ArrowRight && self.props.submenu {
                    group.pending.set(Some(MenuCommand::Open(self.props.index)));
                    return EventResponse::Sink;
                }
                if matches!(key, KeyCode::ArrowLeft | KeyCode::Tab) {
                    if let Some(dismiss) = &self.dismiss {
                        dismiss.dismiss.set(true);
                        ctx.input.set_focus(dismiss.trigger, false);
                    }
                    return if *key == KeyCode::Tab {
                        EventResponse::Bubble
                    } else {
                        EventResponse::Sink
                    };
                }
            }
            if let WidgetEvent::TextInput(c, _) = event {
                if !c.is_control() && count > 0 {
                    let mut search = group.search.borrow_mut();
                    if search.1.elapsed() > Duration::from_millis(750) {
                        search.0.clear();
                    }
                    search.0.push(*c);
                    search.1 = Instant::now();
                    if let Some((_, id, _)) =
                        ids.iter()
                            .cycle()
                            .skip(pos + 1)
                            .take(count)
                            .find(|(_, _, label)| {
                                label.to_lowercase().starts_with(&search.0.to_lowercase())
                            })
                    {
                        ctx.input.set_focus(Some(*id), true);
                    }
                    return EventResponse::Sink;
                }
            }
        }
        self.control.event(ctx, event)
    }
}
#[track_caller]
fn with_menu_context<R>(
    external: Option<MenuContext>,
    children: impl FnOnce(MenuContext) -> R,
) -> R {
    crate::layout::with_state(
        || {
            (
                Rc::new(Cell::new(None::<MenuCommand>)),
                Rc::new(RefCell::new((String::new(), Instant::now()))),
            )
        },
        |hook| {
            let context = external.unwrap_or_else(|| {
                let (pending, search) = hook.borrow().clone();
                MenuContext {
                    ids: Rc::new(RefCell::new(Vec::new())),
                    pending,
                    search,
                }
            });
            children(context)
        },
    )
}
#[derive(Default)]
struct LevelResponse {
    action: Option<u64>,
    submenu: Option<(usize, ControlResponse)>,
}
#[track_caller]
fn menu_list(items: &mut [MenuItem], width: f32, external: Option<MenuContext>) -> LevelResponse {
    with_menu_context(external, |context| {
        let command = context.pending.take();
        let ids = context.ids.clone();
        ids.borrow_mut().clear();
        let mut output = LevelResponse::default();
        let mut radio = None;
        let t = theme();
        Scope::new(context).show(|| {
            crate::column(2., || {
                for (index, item) in items.iter_mut().enumerate() {
                    if matches!(item.kind, MenuKind::Separator) {
                        crate::separator(width - 16.);
                        continue;
                    }
                    if matches!(item.kind, MenuKind::Label) {
                        crate::surface(Color::CLEAR, None, 0., 6., || crate::label(&item.label));
                        continue;
                    }
                    let dom = context::dom();
                    let response = dom.begin_widget::<EntryWidget>(EntryProps {
                        index,
                        disabled: item.disabled,
                        submenu: matches!(item.kind, MenuKind::Submenu(_)),
                    });
                    let id = response.id;
                    let res = ControlResponse {
                        id: Some(id),
                        ..response.into_inner()
                    };
                    if !item.disabled {
                        ids.borrow_mut().push((index, id, item.label.clone()));
                    }
                    let activate =
                        res.clicked || matches!(command,Some(MenuCommand::Activate(i)) if i==index);
                    if activate && !item.disabled {
                        match &mut item.kind {
                            MenuKind::Action => output.action = Some(item.id),
                            MenuKind::Checkbox(checked) => {
                                *checked = !*checked;
                                output.action = Some(item.id);
                            }
                            MenuKind::Radio { group, selected } => {
                                *selected = true;
                                radio = Some((*group, index));
                                output.action = Some(item.id);
                            }
                            MenuKind::Submenu(_) => output.submenu = Some((index, res)),
                            _ => {}
                        }
                    }
                    if matches!(command,Some(MenuCommand::Open(i)) if i==index) {
                        output.submenu = Some((index, res));
                    }
                    crate::width(width - 16., || {
                        crate::surface(
                            if res.focused || res.hovering {
                                t.accent
                            } else {
                                Color::CLEAR
                            },
                            None,
                            t.radius * 0.6,
                            6.,
                            || {
                                crate::row(8., || {
                                    let checked = matches!(
                                        item.kind,
                                        MenuKind::Checkbox(true)
                                            | MenuKind::Radio { selected: true, .. }
                                    );
                                    crate::size(crate::Vec2::splat(14.), || {
                                        if checked {
                                            crate::icon(crate::Icon::Check, 14.);
                                        }
                                    });
                                    let color = if item.disabled {
                                        t.muted_foreground
                                    } else if item.destructive {
                                        t.destructive
                                    } else {
                                        t.popover_foreground
                                    };
                                    yakui::widgets::RenderText::with_style(
                                        &item.label,
                                        t.text_style(14., color, 400),
                                    )
                                    .show();
                                    if let Some(shortcut) = &item.shortcut {
                                        crate::muted(shortcut);
                                    }
                                    if matches!(item.kind, MenuKind::Submenu(_)) {
                                        crate::icon(crate::Icon::ChevronRight, 14.);
                                    }
                                })
                            },
                        )
                    });
                    dom.end_widget::<EntryWidget>(id);
                }
            })
        });
        if let Some((group, index)) = radio {
            for (i, item) in items.iter_mut().enumerate() {
                if let MenuKind::Radio { group: g, selected } = &mut item.kind {
                    if *g == group {
                        *selected = i == index;
                    }
                }
            }
        }
        output
    })
}
/// Menu popup and nested submenus. Returns the stable ID of an activated item.
/// Open the state from an ordinary button response, then render this at the root.
#[track_caller]
pub fn dropdown_menu(state: &mut MenuState, items: &mut [MenuItem]) -> Option<u64> {
    let mut result = LevelResponse::default();
    crate::popover(&mut state.popup, 224., |overlay| {
        result = menu_list(items, 224., None);
        if result.action.is_some() {
            overlay.close();
        }
    });
    if let Some((index, response)) = result.submenu {
        state.path = vec![index];
        state.submenus = vec![OverlayState::default()];
        state.submenus[0].open_from(response);
    }
    let mut action = result.action;
    for depth in 0..state.path.len() {
        let mut nested = &mut *items;
        for &index in state.path.iter().take(depth + 1) {
            let Some(item) = nested.get_mut(index) else {
                return action;
            };
            let MenuKind::Submenu(children) = &mut item.kind else {
                return action;
            };
            nested = children;
        }
        let Some(overlay) = state.submenus.get_mut(depth) else {
            break;
        };
        let mut result = LevelResponse::default();
        crate::popover_on_side(overlay, 224., crate::Side::Right, |overlay| {
            result = menu_list(nested, 224., None);
            if result.action.is_some() {
                overlay.close();
            }
        });
        action = action.or(result.action);
        if let Some((index, response)) = result.submenu {
            state.path.truncate(depth + 1);
            state.submenus.truncate(depth + 1);
            state.path.push(index);
            let mut popup = OverlayState::default();
            popup.open_from(response);
            state.submenus.push(popup);
            break;
        }
    }
    if action.is_some() {
        state.close();
    }
    action
}
/// Select state separates trigger visibility, highlighted navigation and the value.
#[derive(Debug, Default, Clone)]
pub struct SelectState {
    pub selected: Option<usize>,
    pub popup: OverlayState,
    pub query: String,
}
#[track_caller]
pub fn select(state: &mut SelectState, options: &[&str], placeholder: &str, disabled: bool) {
    let label = state
        .selected
        .and_then(|i| options.get(i))
        .copied()
        .unwrap_or(placeholder);
    let response = crate::Button::new(label)
        .variant(crate::Variant::Outline)
        .icon(crate::Icon::ChevronDown)
        .disabled(disabled)
        .show();
    state.popup.trigger = response.id;
    if response.clicked {
        state.popup.toggle_from(response);
    }
}
#[track_caller]
pub fn select_content(state: &mut SelectState, options: &[&str]) {
    let selected = &mut state.selected;
    crate::popover(&mut state.popup, 224., |popup| {
        let mut items: Vec<_> = options
            .iter()
            .enumerate()
            .map(|(i, label)| MenuItem {
                kind: MenuKind::Radio {
                    group: 0,
                    selected: *selected == Some(i),
                },
                ..MenuItem::action(i as u64, *label)
            })
            .collect();
        let result = menu_list(&mut items, 224., None);
        if let Some(id) = result.action {
            *selected = Some(id as usize);
            popup.close();
        }
    });
}
/// Searchable combobox trigger; render its popup separately at the root.
#[track_caller]
pub fn combobox(state: &mut SelectState, options: &[&str], placeholder: &str) {
    select(state, options, placeholder, false)
}
#[track_caller]
pub fn combobox_content(state: &mut SelectState, options: &[&str]) {
    let selected = &mut state.selected;
    let query = &mut state.query;
    crate::popover(&mut state.popup, 260., |popup| {
        with_menu_context(None, |group| {
            Scope::new(group.clone()).show(|| {
                crate::column(8., || {
                    crate::input(query, "Search...");
                    let mut items: Vec<_> = options
                        .iter()
                        .enumerate()
                        .filter(|(_, label)| label.to_lowercase().contains(&query.to_lowercase()))
                        .map(|(i, label)| MenuItem {
                            kind: MenuKind::Radio {
                                group: 0,
                                selected: *selected == Some(i),
                            },
                            ..MenuItem::action(i as u64, *label)
                        })
                        .collect();
                    if items.is_empty() {
                        crate::muted("No results found.");
                    }
                    let result = menu_list(&mut items, 260., Some(group.clone()));
                    if let Some(id) = result.action {
                        *selected = Some(id as usize);
                        popup.close();
                    }
                })
            });
        });
    });
}
/// Inline command palette. Returns the selected command's stable ID.
#[track_caller]
pub fn command(query: &mut String, items: &[MenuItem]) -> Option<u64> {
    let mut result = None;
    let t = theme();
    crate::surface(t.popover, Some(t.border), t.radius * 1.4, 8., || {
        with_menu_context(None, |group| {
            Scope::new(group.clone()).show(|| {
                crate::column(8., || {
                    crate::input(query, "Type a command or search...");
                    let mut filtered: Vec<_> = items
                        .iter()
                        .filter(|item| item.label.to_lowercase().contains(&query.to_lowercase()))
                        .cloned()
                        .collect();
                    if filtered.is_empty() {
                        crate::muted("No results found.");
                    }
                    result = menu_list(&mut filtered, 300., Some(group.clone())).action;
                })
            });
        });
    });
    result
}

#[derive(Debug, Default)]
struct ContextTarget {
    position: Option<crate::Vec2>,
}
impl Widget for ContextTarget {
    type Props<'a> = ();
    type Response = Option<crate::Vec2>;
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, _: ()) -> Self::Response {
        self.position.take()
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }
    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::MouseButtonChanged {
            button: yakui::input::MouseButton::Two,
            down: true,
            inside: true,
            position,
            ..
        } = event
        {
            self.position = Some(*position);
            return EventResponse::Sink;
        }
        EventResponse::Bubble
    }
}
/// Attach a native right-click target to ordinary component content.
#[track_caller]
pub fn context_menu_target(state: &mut MenuState, children: impl FnOnce()) {
    let response = yakui::util::widget_children::<ContextTarget, _>(children, ());
    let id = response.id;
    if let Some(position) = response.into_inner() {
        state.open_from(ControlResponse {
            id: Some(id),
            ..ControlResponse::default()
        });
        state.popup.anchor_position = Some(position);
    }
}
/// Context-menu popup, using the same menu state and item contract as Dropdown Menu.
#[track_caller]
pub fn context_menu(state: &mut MenuState, items: &mut [MenuItem]) -> Option<u64> {
    dropdown_menu(state, items)
}
/// Native select uses the same selection contract; the popup is rendered at the root.
#[track_caller]
pub fn native_select(state: &mut SelectState, options: &[&str], placeholder: &str, disabled: bool) {
    select(state, options, placeholder, disabled)
}
#[derive(Debug, Default, Clone)]
pub struct MenubarState {
    pub active: Option<usize>,
    pub menu: MenuState,
}
/// Menu-bar triggers. Render the selected menu with `dropdown_menu` at the root.
#[track_caller]
pub fn menubar(labels: &[&str], state: &mut MenubarState) {
    crate::surface(
        theme().background,
        Some(theme().border),
        theme().radius,
        3.,
        || {
            crate::row(0., || {
                for (i, label) in labels.iter().enumerate() {
                    let response = crate::Button::new(*label)
                        .variant(if state.active == Some(i) && state.menu.popup.open {
                            crate::Variant::Secondary
                        } else {
                            crate::Variant::Ghost
                        })
                        .size(crate::ButtonSize::Sm)
                        .show();
                    if response.clicked {
                        state.active = Some(i);
                        state.menu.open_from(response);
                    }
                }
            })
        },
    );
}
/// Primary navigation actions return an index for the application's router.
#[track_caller]
pub fn navigation_menu(labels: &[&str], selected: &mut usize) -> Option<usize> {
    let mut result = None;
    crate::row(4., || {
        for (i, label) in labels.iter().enumerate() {
            if crate::Button::new(*label)
                .variant(if *selected == i {
                    crate::Variant::Secondary
                } else {
                    crate::Variant::Ghost
                })
                .show()
                .clicked
            {
                *selected = i;
                result = Some(i);
            }
        }
    });
    result
}
