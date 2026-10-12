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
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
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
    pub icon: Option<crate::Icon>,
    pub inset: bool,
    pub disabled: bool,
    pub destructive: bool,
    /// Action items close by default; checkbox/radio items stay open.
    pub close_on_select: bool,
    pub kind: MenuKind,
}
impl MenuItem {
    pub fn action(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            shortcut: None,
            icon: None,
            inset: false,
            disabled: false,
            destructive: false,
            close_on_select: true,
            kind: MenuKind::Action,
        }
    }
    pub fn checkbox(id: u64, label: impl Into<String>, checked: bool) -> Self {
        Self {
            kind: MenuKind::Checkbox(checked),
            close_on_select: false,
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
    pub fn icon(mut self, icon: crate::Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }
    pub fn label(text: impl Into<String>) -> Self {
        Self {
            kind: MenuKind::Label,
            ..Self::action(0, text)
        }
    }
    pub fn radio(id: u64, label: impl Into<String>, group: u64, selected: bool) -> Self {
        Self {
            kind: MenuKind::Radio { group, selected },
            close_on_select: false,
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
    sub_pointer: Vec<bool>,
    pointer_open: bool,
    pending_focus: Option<MenuInitialFocus>,
}
impl MenuState {
    pub fn open_from(&mut self, response: ControlResponse) {
        self.path.clear();
        self.submenus.clear();
        self.sub_pointer.clear();
        self.pointer_open = false;
        self.pending_focus = None;
        self.popup.open_from(response);
    }
    pub fn close(&mut self) {
        self.popup.close();
        self.path.clear();
        self.submenus.clear();
        self.sub_pointer.clear();
        self.pending_focus = None;
    }
}
#[derive(Debug, Clone, Copy)]
enum MenuInitialFocus {
    Pointer,
    First,
    Last,
    Item(usize),
}
#[derive(Debug, Clone, Copy)]
enum MenuCommand {
    Activate(usize),
    Open(usize),
    CloseSubmenus,
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
    submenu_level: bool,
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
                submenu_level: false,
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
            ring: Color::CLEAR,
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
        if matches!(event, WidgetEvent::MouseEnter) {
            ctx.input.set_focus(Some(ctx.dom.current()), false);
            if let Some(group) = &self.context {
                group.pending.set(Some(if self.props.submenu {
                    MenuCommand::Open(self.props.index)
                } else {
                    MenuCommand::CloseSubmenus
                }));
            }
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
                if *key == KeyCode::Tab || (*key == KeyCode::ArrowLeft && self.props.submenu_level)
                {
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
            if matches!(
                event,
                WidgetEvent::KeyChanged {
                    key: KeyCode::ArrowLeft | KeyCode::ArrowRight,
                    ..
                }
            ) {
                return EventResponse::Sink;
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
    close_submenus: bool,
    close_on_select: bool,
    first: Option<WidgetId>,
    last: Option<WidgetId>,
}
#[track_caller]
fn menu_list(items: &mut [MenuItem], width: f32, external: Option<MenuContext>) -> LevelResponse {
    with_menu_context(external, |context| {
        let command = context.pending.take();
        let ids = context.ids.clone();
        ids.borrow_mut().clear();
        let mut output = LevelResponse {
            close_submenus: matches!(command, Some(MenuCommand::CloseSubmenus)),
            ..Default::default()
        };
        let mut radio = None;
        let t = theme();
        let translucent = yakui::util::read_scope::<MenuTranslucent>().is_some();
        Scope::new(context).show(|| {
            crate::column(0., || {
                for (index, item) in items.iter_mut().enumerate() {
                    if matches!(item.kind, MenuKind::Separator) {
                        menu_separator(width);
                        continue;
                    }
                    if matches!(item.kind, MenuKind::Label) {
                        yakui::util::widget_children::<MenuRow, _>(
                            || {
                                let mut style = t.text_style(
                                    12.,
                                    t.muted_foreground,
                                    if yakui::util::read_scope::<SelectRows>().is_some() {
                                        400
                                    } else {
                                        500
                                    },
                                );
                                style.line_height_override = Some(16.);
                                yakui::widgets::RenderText::with_style(&item.label, style).show();
                            },
                            MenuRowProps {
                                width,
                                inset: item.inset,
                                indicator: false,
                                minimum_height: 16.,
                            },
                        );
                        continue;
                    }
                    let dom = context::dom();
                    let response = dom.begin_widget::<EntryWidget>(EntryProps {
                        index,
                        disabled: item.disabled,
                        submenu: matches!(item.kind, MenuKind::Submenu(_)),
                        submenu_level: yakui::util::read_scope::<MenuRootLevel>()
                            .is_some_and(|level| level.0),
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
                        output.close_on_select = item.close_on_select;
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
                    let active = res.focused
                        || res.hovering
                        || yakui::util::read_scope::<MenuOpenItem>()
                            .is_some_and(|open| open.0 == Some(index));
                    let fill = if active && translucent {
                        t.foreground.with_alpha(0.1)
                    } else if active && item.destructive {
                        t.destructive.with_alpha(if t.dark { 0.2 } else { 0.1 })
                    } else if active {
                        t.accent
                    } else {
                        Color::CLEAR
                    };
                    let color = if item.destructive && translucent {
                        t.accent_foreground
                    } else if item.destructive {
                        t.destructive
                    } else if active {
                        t.accent_foreground
                    } else {
                        t.popover_foreground
                    };
                    let color = crate::layout::resolve_color(color.with_alpha(if item.disabled {
                        0.5
                    } else {
                        1.
                    }));
                    let indicator =
                        matches!(item.kind, MenuKind::Checkbox(_) | MenuKind::Radio { .. });
                    crate::surface(fill, None, t.radius * 0.6, 0., || {
                        yakui::util::widget_children::<MenuRow, _>(
                            || {
                                yakui::util::widget_children::<MenuLeading, _>(
                                    || {
                                        if let Some(icon) = item.icon {
                                            crate::icon_colored(icon, 16., color);
                                        }
                                        let text = || {
                                            yakui::widgets::RenderText::with_style(
                                                &item.label,
                                                t.text_style(14., color, 400),
                                            )
                                            .show();
                                        };
                                        if yakui::util::read_scope::<SelectRows>().is_some() {
                                            crate::select::single_line(text);
                                        } else {
                                            text();
                                        }
                                    },
                                    item.icon.is_some(),
                                );
                                if indicator {
                                    crate::size(crate::Vec2::splat(16.), || {
                                        if matches!(
                                            item.kind,
                                            MenuKind::Checkbox(true)
                                                | MenuKind::Radio { selected: true, .. }
                                        ) {
                                            crate::icon_colored(crate::Icon::Check, 16., color);
                                        }
                                    });
                                } else if let Some(shortcut) = &item.shortcut {
                                    let mut style = t.text_style(
                                        12.,
                                        if active { color } else { t.muted_foreground },
                                        400,
                                    );
                                    style.line_height_override = Some(16.);
                                    style.attrs = yakui::cosmic_text::AttrsOwned::new(
                                        &style.attrs.as_attrs().letter_spacing(0.1),
                                    );
                                    yakui::widgets::RenderText::with_style(shortcut, style).show();
                                } else if matches!(item.kind, MenuKind::Submenu(_)) {
                                    crate::icon_colored(crate::Icon::ChevronRight, 16., color);
                                }
                            },
                            MenuRowProps {
                                width,
                                inset: item.inset,
                                indicator,
                                minimum_height: 20.,
                            },
                        );
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
        output.first = ids.borrow().first().map(|entry| entry.1);
        output.last = ids.borrow().last().map(|entry| entry.1);
        output
    })
}
/// Menu popup and nested submenus. Returns the stable ID of an activated item.
/// Open the state from an ordinary button response, then render this at the root.
#[track_caller]
pub fn dropdown_menu(state: &mut MenuState, items: &mut [MenuItem]) -> Option<u64> {
    dropdown_menu_with_options(state, items, MenuOptions::default())
}
fn natural_menu_width(items: &[MenuItem], minimum: f32) -> f32 {
    let t = theme();
    let fonts = context::dom().get_global_or_init(yakui::font::Fonts::default);
    fonts.with_system(|system| {
        let mut buffer =
            yakui::cosmic_text::Buffer::new(system, yakui::cosmic_text::Metrics::new(14., 20.));
        let mut measure = |text: &str, size: f32, weight: u16, tracking: f32| {
            let style = t.text_style(size, t.popover_foreground, weight);
            buffer.set_metrics_and_size(system, style.to_metrics(1.), None, None);
            buffer.set_text(
                system,
                text,
                &style.attrs.as_attrs().letter_spacing(tracking),
                yakui::cosmic_text::Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(system, true);
            buffer
                .layout_runs()
                .map(|run| run.line_w)
                .fold(0., f32::max)
        };
        let mut width = minimum;
        // The child popup has its own intrinsic width, independent of the root's
        // explicit width. Measure using Yakui's actual font system so long labels,
        // icons and shortcuts fit without a second text renderer or guessed widths.
        for item in items {
            if matches!(item.kind, MenuKind::Separator) {
                continue;
            }
            let label = matches!(item.kind, MenuKind::Label);
            let leading = measure(
                &item.label,
                if label { 12. } else { 14. },
                if label { 500 } else { 400 },
                0.,
            ) + if item.icon.is_some() { 22. } else { 0. };
            let reserved = if matches!(item.kind, MenuKind::Checkbox(_) | MenuKind::Radio { .. }) {
                32.
            } else if let Some(shortcut) = &item.shortcut {
                12. + measure(shortcut, 12., 400, 0.1)
            } else if matches!(item.kind, MenuKind::Submenu(_)) {
                28.
            } else {
                6.
            };
            width = width.max(8. + if item.inset { 28. } else { 6. } + leading + reserved);
        }
        width.ceil()
    })
}
/// Menu placement and width; item geometry follows the scoped Nova tokens.
#[derive(Debug, Clone, Copy)]
pub struct MenuOptions {
    /// Explicit outer width. Zero uses the trigger width with a 128px minimum.
    pub width: f32,
    pub side: crate::Side,
    pub align: crate::PopoverAlign,
    pub side_offset: f32,
    pub align_offset: f32,
}
impl Default for MenuOptions {
    fn default() -> Self {
        Self {
            width: 0.,
            side: crate::Side::Bottom,
            align: crate::PopoverAlign::Start,
            side_offset: 4.,
            align_offset: 0.,
        }
    }
}
#[track_caller]
pub fn dropdown_menu_with_options(
    state: &mut MenuState,
    items: &mut [MenuItem],
    options: MenuOptions,
) -> Option<u64> {
    let pending_focus = state.pending_focus.take();
    let open_index = state.path.first().copied();
    let mut result = LevelResponse::default();
    crate::overlay::menu_popup(
        &mut state.popup,
        crate::PopoverOptions {
            width: options.width,
            side: options.side,
            align: options.align,
            side_offset: options.side_offset,
            align_offset: options.align_offset,
        },
        false,
        true,
        |overlay| {
            result = menu_root(
                items,
                if options.width > 0. {
                    (options.width - 8.).max(0.)
                } else {
                    0.
                },
                if state.pointer_open {
                    MenuInitialFocus::Pointer
                } else {
                    MenuInitialFocus::First
                },
                pending_focus,
                false,
                open_index,
            );
            if result.action.is_some() && result.close_on_select {
                overlay.close();
            }
        },
    );
    if !state.popup.open {
        state.close();
        return result.action;
    }
    if result.close_submenus {
        state.path.clear();
        state.submenus.clear();
        state.sub_pointer.clear();
    }
    if let Some((index, response)) = result.submenu {
        state.path = vec![index];
        state.submenus = vec![OverlayState::default()];
        state.sub_pointer = vec![!response.focus_visible];
        state.submenus[0].open_from(response);
    }
    let mut action = result.action;
    let mut close = result.close_on_select;
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
        let width = natural_menu_width(nested, 96.);
        let open_index = state.path.get(depth + 1).copied();
        crate::overlay::menu_popup(
            overlay,
            crate::PopoverOptions {
                width,
                side: crate::Side::Right,
                align_offset: -3.,
                side_offset: 0.,
                ..Default::default()
            },
            false,
            false,
            |overlay| {
                result = menu_root(
                    nested,
                    (width - 8.).max(0.),
                    if state.sub_pointer.get(depth).copied().unwrap_or(false) {
                        MenuInitialFocus::Pointer
                    } else {
                        MenuInitialFocus::First
                    },
                    None,
                    true,
                    open_index,
                );
                if result.action.is_some() && result.close_on_select {
                    overlay.close();
                }
            },
        );
        if !overlay.open && result.action.is_none() {
            state.path.truncate(depth);
            state.submenus.truncate(depth);
            state.sub_pointer.truncate(depth);
            break;
        }
        if result.close_submenus {
            state.path.truncate(depth + 1);
            state.submenus.truncate(depth + 1);
            state.sub_pointer.truncate(depth + 1);
        }
        if result.action.is_some() {
            close = result.close_on_select;
        }
        action = action.or(result.action);
        if let Some((index, response)) = result.submenu {
            state.path.truncate(depth + 1);
            state.submenus.truncate(depth + 1);
            state.sub_pointer.truncate(depth + 1);
            state.path.push(index);
            let mut popup = OverlayState::default();
            popup.open_from(response);
            state.submenus.push(popup);
            state.sub_pointer.push(!response.focus_visible);
            break;
        }
    }
    if action.is_some() && close {
        state.close();
    }
    action
}
/// Select state separates trigger visibility, highlighted navigation and the value.
#[derive(Debug, Default, Clone)]
pub struct SelectState {
    /// Selected application value; positional shorthands use option indices.
    pub selected: Option<usize>,
    pub popup: OverlayState,
    pub query: String,
    pub(crate) request_last: Option<bool>,
}

#[derive(Debug)]
struct MenuLeading(bool);
impl Widget for MenuLeading {
    type Props<'a> = bool;
    type Response = ();
    fn new() -> Self {
        Self(false)
    }
    fn update(&mut self, icon: bool) {
        self.0 = icon;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: yakui::Constraints) -> crate::Vec2 {
        let children = &ctx.dom.get_current().children;
        let icon = if self.0 {
            ctx.calculate_layout(
                children[0],
                yakui::Constraints::loose(crate::Vec2::splat(16.)),
            )
        } else {
            crate::Vec2::ZERO
        };
        let offset = if self.0 { icon.x + 6. } else { 0. };
        let text = ctx.calculate_layout(
            children[usize::from(self.0)],
            yakui::Constraints::loose(crate::Vec2::new(
                (constraints.max.x - offset).max(0.),
                constraints.max.y,
            )),
        );
        let size = constraints.constrain(crate::Vec2::new(offset + text.x, icon.y.max(text.y)));
        if self.0 {
            ctx.layout
                .set_pos(children[0], crate::Vec2::new(0., (size.y - icon.y) / 2.));
        }
        ctx.layout.set_pos(
            children[usize::from(self.0)],
            crate::Vec2::new(offset, (size.y - text.y) / 2.),
        );
        size
    }
}
#[derive(Debug, Clone, Copy)]
struct MenuRowProps {
    width: f32,
    inset: bool,
    indicator: bool,
    minimum_height: f32,
}
#[derive(Debug)]
struct MenuRow(MenuRowProps);
impl Widget for MenuRow {
    type Props<'a> = MenuRowProps;
    type Response = ();
    fn new() -> Self {
        Self(MenuRowProps {
            width: 0.,
            inset: false,
            indicator: false,
            minimum_height: 20.,
        })
    }
    fn update(&mut self, props: MenuRowProps) {
        self.0 = props;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: yakui::Constraints) -> crate::Vec2 {
        let width = if self.0.width > 0. {
            self.0.width.min(input.max.x)
        } else {
            input.max.x
        };
        let left_padding = if self.0.inset { 28. } else { 6. };
        let right_padding = if self.0.indicator { 8. } else { 6. };
        let children = &ctx.dom.get_current().children;
        let mut suffix = crate::Vec2::ZERO;
        if let Some(&child) = children.get(1) {
            suffix = ctx.calculate_layout(
                child,
                yakui::Constraints::loose(crate::Vec2::new(
                    (width - left_padding - right_padding).max(0.),
                    input.max.y,
                )),
            );
        }
        let reserved = if self.0.indicator {
            32.
        } else {
            right_padding
                + if children.len() > 1 {
                    suffix.x + 6.
                } else {
                    0.
                }
        };
        let leading = ctx.calculate_layout(
            children[0],
            yakui::Constraints::loose(crate::Vec2::new(
                (width - left_padding - reserved).max(0.),
                input.max.y,
            )),
        );
        let size = input.constrain(crate::Vec2::new(
            width,
            leading.y.max(suffix.y).max(self.0.minimum_height) + 8.,
        ));
        ctx.layout.set_pos(
            children[0],
            crate::Vec2::new(left_padding, (size.y - leading.y) / 2.),
        );
        if let Some(&child) = children.get(1) {
            ctx.layout.set_pos(
                child,
                crate::Vec2::new(width - right_padding - suffix.x, (size.y - suffix.y) / 2.),
            );
        }
        size
    }
}
#[track_caller]
fn menu_separator(width: f32) {
    let t = theme();
    let color =
        crate::layout::resolve_color(if yakui::util::read_scope::<MenuTranslucent>().is_some() {
            t.foreground.with_alpha(0.05)
        } else {
            t.border
        });
    yakui::util::widget::<MenuSeparator>((width, color));
}
#[derive(Debug)]
struct MenuSeparator {
    width: f32,
    color: Color,
}
impl Widget for MenuSeparator {
    type Props<'a> = (f32, Color);
    type Response = ();
    fn new() -> Self {
        Self {
            width: 0.,
            color: Color::CLEAR,
        }
    }
    fn update(&mut self, (width, color): Self::Props<'_>) {
        self.width = width;
        self.color = color;
    }
    fn layout(&self, _: LayoutContext<'_>, constraints: yakui::Constraints) -> crate::Vec2 {
        let width = if self.width > 0. {
            self.width.min(constraints.max.x)
        } else {
            constraints.max.x
        };
        constraints.constrain(crate::Vec2::new(width, 9.))
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let mut line = yakui::paint::PaintRect::new(yakui::Rect::from_pos_size(
            rect.pos() + crate::Vec2::new(-4., 4.),
            crate::Vec2::new(rect.size().x + 8., 1.),
        ));
        line.color = self.color;
        line.add(ctx.paint);
    }
}

#[derive(Debug)]
struct MenuRootLevel(bool);
#[derive(Debug)]
struct MenuOpenItem(Option<usize>);
#[derive(Debug)]
struct MenuTranslucent;
#[derive(Debug)]
struct MenuRootProps {
    context: MenuContext,
    initial: MenuInitialFocus,
    requested: Option<MenuInitialFocus>,
    submenu: bool,
}
#[derive(Debug)]
struct MenuRoot {
    props: Option<MenuRootProps>,
    focused_once: Cell<bool>,
    requested: Cell<Option<MenuInitialFocus>>,
    dismiss: Option<Rc<crate::overlay::DismissContext>>,
}
impl Widget for MenuRoot {
    type Props<'a> = MenuRootProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: None,
            focused_once: Cell::new(false),
            requested: Cell::new(None),
            dismiss: None,
        }
    }
    fn update(&mut self, props: MenuRootProps) {
        self.requested.set(props.requested);
        self.dismiss = yakui::util::read_scope();
        self.props = Some(props);
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: yakui::Constraints) -> crate::Vec2 {
        let size = self.default_layout(
            LayoutContext {
                dom: ctx.dom,
                input: ctx.input,
                layout: ctx.layout,
                paint: ctx.paint,
            },
            constraints,
        );
        let props = self.props.as_ref().unwrap();
        let first_layout = !self.focused_once.replace(true);
        let requested = self.requested.take().or_else(|| {
            if first_layout {
                Some(props.initial)
            } else {
                None
            }
        });
        if let Some(requested) = requested {
            let ids = props.context.ids.borrow();
            let target = match requested {
                MenuInitialFocus::Pointer => Some(ctx.dom.current()),
                MenuInitialFocus::First => ids.first().map(|e| e.1),
                MenuInitialFocus::Last => ids.last().map(|e| e.1),
                MenuInitialFocus::Item(index) => ids
                    .iter()
                    .find(|e| e.0 == index)
                    .or_else(|| ids.first())
                    .map(|e| e.1),
            };
            ctx.input
                .set_focus(target, !matches!(requested, MenuInitialFocus::Pointer));
        }
        size
    }
    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::POINTER
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::FOCUSED_KEYBOARD
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        let props = self.props.as_ref().unwrap();
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            return EventResponse::Sink;
        }
        if let WidgetEvent::KeyChanged {
            key, down: true, ..
        } = event
        {
            let ids = props.context.ids.borrow();
            let target = match key {
                KeyCode::ArrowDown => ids.first(),
                KeyCode::ArrowUp => ids.last(),
                _ => None,
            };
            if let Some((_, target, _)) = target {
                ctx.input.set_focus(Some(*target), true);
                return EventResponse::Sink;
            }
            if *key == KeyCode::Tab || (*key == KeyCode::ArrowLeft && props.submenu) {
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
            if matches!(key, KeyCode::Enter | KeyCode::Space) {
                return EventResponse::Sink;
            }
        }
        if matches!(
            event,
            WidgetEvent::KeyChanged {
                key: KeyCode::ArrowLeft | KeyCode::ArrowRight,
                ..
            }
        ) {
            return EventResponse::Sink;
        }
        if let WidgetEvent::TextInput(c, _) = event {
            if !c.is_control() {
                if let Some((_, target, _)) =
                    props.context.ids.borrow().iter().find(|(_, _, text)| {
                        text.to_lowercase()
                            .starts_with(&c.to_lowercase().to_string())
                    })
                {
                    ctx.input.set_focus(Some(*target), true);
                }
                return EventResponse::Sink;
            }
        }
        EventResponse::Bubble
    }
}
#[track_caller]
fn menu_root(
    items: &mut [MenuItem],
    width: f32,
    initial: MenuInitialFocus,
    requested: Option<MenuInitialFocus>,
    submenu: bool,
    open_index: Option<usize>,
) -> LevelResponse {
    with_menu_context(None, |group| {
        let mut result = LevelResponse::default();
        yakui::util::widget_children::<MenuRoot, _>(
            || {
                yakui::widgets::Scrollable::vertical().show(|| {
                    Scope::new(MenuTranslucent).show(|| {
                        Scope::new(MenuRootLevel(submenu)).show(|| {
                            Scope::new(MenuOpenItem(open_index))
                                .show(|| result = menu_list(items, width, Some(group.clone())));
                        });
                    });
                });
            },
            MenuRootProps {
                context: group.clone(),
                initial,
                requested,
                submenu,
            },
        );
        result
    })
}
#[derive(Debug, Clone, Copy)]
enum TriggerCommand {
    First,
    Last,
    Close,
}
#[derive(Debug)]
pub(crate) struct MenuTriggerContext {
    open: bool,
    pending: Rc<Cell<Option<TriggerCommand>>>,
}
impl MenuTriggerContext {
    pub(crate) fn key(&self, event: &WidgetEvent) -> Option<EventResponse> {
        if let WidgetEvent::KeyChanged { key, down, .. } = event {
            let command = match key {
                KeyCode::ArrowDown => TriggerCommand::First,
                KeyCode::ArrowUp => TriggerCommand::Last,
                KeyCode::Escape | KeyCode::Tab if self.open => TriggerCommand::Close,
                _ => return None,
            };
            if *down {
                self.pending.set(Some(command));
            }
            return Some(if *key == KeyCode::Tab {
                EventResponse::Bubble
            } else {
                EventResponse::Sink
            });
        }
        None
    }
}
/// Compose an ordinary Button as a dropdown trigger. Pointer opening leaves the
/// rows unhighlighted; ArrowDown/Up opens and focuses the first/last enabled row.
#[track_caller]
pub fn dropdown_menu_trigger(
    state: &mut MenuState,
    children: impl FnOnce() -> ControlResponse,
) -> ControlResponse {
    let mut response = ControlResponse::default();
    let command = crate::layout::with_state(
        || Rc::new(Cell::new(None::<TriggerCommand>)),
        |hook| {
            let pending = hook.borrow().clone();
            let command = pending.take();
            Scope::new(MenuTriggerContext {
                open: state.popup.open,
                pending,
            })
            .show(|| {
                response = children();
            });
            command
        },
    );
    state.popup.trigger = response.id;
    if response.clicked {
        if state.popup.open {
            state.close();
        } else {
            state.open_from(response);
            state.pointer_open = !response.focus_visible;
        }
    }
    match command {
        Some(TriggerCommand::First | TriggerCommand::Last) => {
            state.open_from(response);
            state.pending_focus = Some(if matches!(command, Some(TriggerCommand::Last)) {
                MenuInitialFocus::Last
            } else {
                MenuInitialFocus::First
            });
        }
        Some(TriggerCommand::Close) => state.close(),
        None => {}
    }
    response
}
/// A single-line select using positional values. Render select_content at the root.
#[track_caller]
pub fn select(state: &mut SelectState, options: &[&str], placeholder: &str, disabled: bool) {
    let items: Vec<_> = options
        .iter()
        .enumerate()
        .map(|(value, label)| crate::SelectItem::new(value, *label))
        .collect();
    crate::Select::new(&items)
        .placeholder(placeholder)
        .disabled(disabled)
        .show(state);
}
/// Popup for the positional shorthand. New applications can use SelectContent directly.
#[track_caller]
pub fn select_content(state: &mut SelectState, options: &[&str]) {
    let items: Vec<_> = options
        .iter()
        .enumerate()
        .map(|(value, label)| crate::SelectItem::new(value, *label))
        .collect();
    crate::SelectContent::new(&items)
        .align_item_with_trigger(false)
        .width(224.)
        .show(state);
}
#[derive(Debug)]
struct SelectRows;
#[track_caller]
pub(crate) fn select_list(
    items: &mut [MenuItem],
    selected: Option<usize>,
    last: bool,
) -> Option<u64> {
    let mut action = None;
    Scope::new(SelectRows).show(|| {
        action = menu_root(
            items,
            0.,
            selected.map(MenuInitialFocus::Item).unwrap_or(if last {
                MenuInitialFocus::Last
            } else {
                MenuInitialFocus::First
            }),
            None,
            false,
            None,
        )
        .action;
    });
    action
}
#[track_caller]
pub(crate) fn select_trigger(
    state: &mut SelectState,
    children: impl FnOnce() -> ControlResponse,
) -> ControlResponse {
    let mut response = ControlResponse::default();
    let command = crate::layout::with_state(
        || Rc::new(Cell::new(None::<TriggerCommand>)),
        |hook| {
            let pending = hook.borrow().clone();
            let command = pending.take();
            Scope::new(MenuTriggerContext {
                open: state.popup.open,
                pending,
            })
            .show(|| response = children());
            command
        },
    );
    state.popup.trigger = response.id;
    if response.clicked {
        state.popup.toggle_from(response);
        state.request_last = Some(false);
    }
    match command {
        Some(TriggerCommand::First | TriggerCommand::Last) => {
            state.popup.open_from(response);
            state.request_last = Some(matches!(command, Some(TriggerCommand::Last)));
        }
        Some(TriggerCommand::Close) => state.popup.close(),
        None => {}
    }
    response
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
                    let result = menu_list(&mut items, 244., Some(group.clone()));
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
                    result = menu_list(&mut filtered, 284., Some(group.clone())).action;
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
