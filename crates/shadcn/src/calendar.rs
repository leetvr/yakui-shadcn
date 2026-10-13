//! Gregorian calendar state, day selection, and keyboard navigation without a web runtime.
use crate::interaction::{Control, ControlWidget};
use crate::{Button, ButtonSize, ControlResponse, Icon, Variant, theme};
use std::{cell::Cell, rc::Rc};
use yakui::{
    Constraints, Vec2, context,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::KeyCode,
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
    widgets::Scope,
};
/// A validated date in the proleptic Gregorian calendar (years 1 through 9999).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}
impl Date {
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=9999).contains(&year)
            || !(1..=12).contains(&month)
            || day == 0
            || day > month_days(year, month)
        {
            None
        } else {
            Some(Self { year, month, day })
        }
    }
    pub fn year(self) -> i32 {
        self.year
    }
    pub fn month(self) -> u8 {
        self.month
    }
    pub fn day(self) -> u8 {
        self.day
    }
    /// Sunday is zero, matching the default upstream calendar.
    pub fn weekday(self) -> u8 {
        (self.days() + 4).rem_euclid(7) as u8
    }
    pub fn checked_add_days(self, days: i32) -> Option<Self> {
        Self::from_days(self.days().checked_add(days)?)
    }
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
    fn days(self) -> i32 {
        let y = self.year - i32::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yy = y - era * 400;
        let m = self.month as i32 + if self.month > 2 { -3 } else { 9 };
        let doy = (153 * m + 2) / 5 + self.day as i32 - 1;
        era * 146097 + yy * 365 + yy / 4 - yy / 100 + doy - 719468
    }
    fn from_days(days: i32) -> Option<Self> {
        let z = days.checked_add(719468)?;
        let era = z.div_euclid(146097);
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let mut y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = mp + if mp < 10 { 3 } else { -9 };
        y += i32::from(month <= 2);
        Self::new(y, month as u8, day as u8)
    }
}
fn month_days(year: i32, month: u8) -> u8 {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
/// Documented selection modes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CalendarMode {
    #[default]
    Single,
    Multiple,
    Range,
}
/// Application-owned month, focus date, selection and optional bounds.
#[derive(Debug, Clone)]
pub struct CalendarState {
    pub year: i32,
    pub month: u8,
    pub focused: Date,
    pub mode: CalendarMode,
    pub selected: Vec<Date>,
    pub minimum: Option<Date>,
    pub maximum: Option<Date>,
}
impl CalendarState {
    pub fn new(month: Date) -> Self {
        Self {
            year: month.year,
            month: month.month,
            focused: month,
            mode: CalendarMode::Single,
            selected: Vec::new(),
            minimum: None,
            maximum: None,
        }
    }
    pub fn selected_date(&self) -> Option<Date> {
        self.selected.first().copied()
    }
    pub fn select(&mut self, date: Date) {
        if !self.enabled(date) {
            return;
        }
        self.focused = date;
        match self.mode {
            CalendarMode::Single => self.selected = vec![date],
            CalendarMode::Multiple => {
                if let Some(i) = self.selected.iter().position(|d| *d == date) {
                    self.selected.remove(i);
                } else {
                    self.selected.push(date);
                    self.selected.sort();
                }
            }
            CalendarMode::Range => {
                if self.selected.len() != 1 {
                    self.selected = vec![date];
                } else {
                    self.selected.push(date);
                    self.selected.sort();
                }
            }
        }
    }
    pub fn change_month(&mut self, delta: i32) {
        let month = (self.year * 12 + self.month as i32 - 1 + delta).clamp(12, 9999 * 12 + 11);
        self.year = month.div_euclid(12);
        self.month = (month.rem_euclid(12) + 1) as u8;
    }
    fn enabled(&self, date: Date) -> bool {
        self.minimum.is_none_or(|min| date >= min) && self.maximum.is_none_or(|max| date <= max)
    }
    fn selected(&self, date: Date) -> bool {
        if self.mode == CalendarMode::Range && self.selected.len() == 2 {
            date >= self.selected[0] && date <= self.selected[1]
        } else {
            self.selected.contains(&date)
        }
    }
}
#[derive(Debug, Clone)]
struct CalendarContext {
    pending: Rc<Cell<Option<Date>>>,
}
#[derive(Debug, Clone, Copy)]
struct DayProps {
    date: Date,
    disabled: bool,
    tab_stop: bool,
    move_focus: bool,
}
#[derive(Debug)]
struct DayWidget {
    props: DayProps,
    control: ControlWidget,
    context: Option<Rc<CalendarContext>>,
}
impl Widget for DayWidget {
    type Props<'a> = DayProps;
    type Response = ControlResponse;
    fn new() -> Self {
        Self {
            props: DayProps {
                date: Date::new(1970, 1, 1).unwrap(),
                disabled: false,
                tab_stop: false,
                move_focus: false,
            },
            control: ControlWidget::new(),
            context: None,
        }
    }
    fn update(&mut self, props: DayProps) -> ControlResponse {
        self.props = props;
        self.context = yakui::util::read_scope::<CalendarContext>();
        let t = theme();
        self.control.update(Control {
            disabled: props.disabled,
            radius: t.radius * 0.8,
            ring: t.ring,
            focus_size: None,
        })
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        if self.props.move_focus {
            ctx.input.set_focus(Some(ctx.dom.current()), true);
        }
        self.default_layout(ctx, constraints)
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        self.control.paint(ctx)
    }
    fn event_interest(&self) -> EventInterest {
        self.control.event_interest()
    }
    fn focus_policy(&self) -> FocusPolicy {
        if self.props.disabled {
            FocusPolicy::empty()
        } else {
            FocusPolicy::POINTER
                | if self.props.tab_stop {
                    FocusPolicy::SEQUENTIAL
                } else {
                    FocusPolicy::empty()
                }
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if !self.props.disabled {
            if let WidgetEvent::KeyChanged {
                key, down: true, ..
            } = event
            {
                let delta = match key {
                    KeyCode::ArrowLeft => Some(-1),
                    KeyCode::ArrowRight => Some(1),
                    KeyCode::ArrowUp => Some(-7),
                    KeyCode::ArrowDown => Some(7),
                    KeyCode::Home => Some(-(self.props.date.weekday() as i32)),
                    KeyCode::End => Some(6 - self.props.date.weekday() as i32),
                    _ => None,
                };
                if let Some(delta) = delta {
                    if let (Some(group), Some(date)) =
                        (&self.context, self.props.date.checked_add_days(delta))
                    {
                        group.pending.set(Some(date));
                    }
                    return EventResponse::Sink;
                }
            }
        }
        self.control.event(ctx, event)
    }
}
/// Native calendar with pointer selection and roving arrow/Home/End keyboard focus.
/// Returns true when the selected date(s) changed.
#[track_caller]
pub fn calendar(state: &mut CalendarState) -> bool {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    state.year = state.year.clamp(1, 9999);
    state.month = state.month.clamp(1, 12);
    crate::layout::with_state(
        || Rc::new(Cell::new(None::<Date>)),
        |hook| {
            let pending = hook.borrow().clone();
            let moved = pending.take().filter(|date| state.enabled(*date));
            if let Some(date) = moved {
                state.focused = date;
                state.year = date.year;
                state.month = date.month;
            }
            let context = CalendarContext { pending };
            let t = theme();
            let previous = state.selected.clone();
            Scope::new(context).show(|| {
                crate::surface(t.background, Some(t.border), t.radius, 8., || {
                    crate::column(8., || {
                        crate::row(4., || {
                            if Button::new("")
                                .variant(Variant::Ghost)
                                .size(ButtonSize::IconSm)
                                .icon(Icon::ChevronLeft)
                                .show()
                                .clicked
                            {
                                state.change_month(-1);
                            }
                            crate::width(132., || {
                                crate::label(format!(
                                    "{} {}",
                                    MONTHS[state.month as usize - 1],
                                    state.year
                                ))
                            });
                            if Button::new("")
                                .variant(Variant::Ghost)
                                .size(ButtonSize::IconSm)
                                .icon(Icon::ChevronRight)
                                .show()
                                .clicked
                            {
                                state.change_month(1);
                            }
                        });
                        crate::row(0., || {
                            for day in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                                crate::size(Vec2::splat(28.), || {
                                    yakui::center(|| crate::muted(day));
                                });
                            }
                        });
                        let first = Date::new(state.year, state.month, 1).unwrap();
                        let offset = first.weekday() as i32;
                        let cells =
                            ((offset + month_days(state.year, state.month) as i32 + 6) / 7) * 7;
                        for week in 0..cells / 7 {
                            crate::row(0., || {
                                for day in 0..7 {
                                    let Some(date) =
                                        first.checked_add_days(week * 7 + day - offset)
                                    else {
                                        crate::size(Vec2::splat(28.), || {});
                                        continue;
                                    };
                                    let disabled = !state.enabled(date);
                                    let selected = state.selected(date);
                                    let dom = context::dom();
                                    let response = dom.begin_widget::<DayWidget>(DayProps {
                                        date,
                                        disabled,
                                        tab_stop: state.focused == date,
                                        move_focus: moved == Some(date),
                                    });
                                    let id = response.id;
                                    let res = response.into_inner();
                                    if res.clicked {
                                        state.select(date);
                                    }
                                    crate::size(Vec2::splat(28.), || {
                                        crate::surface(
                                            if selected {
                                                t.primary
                                            } else if res.hovering {
                                                t.accent
                                            } else {
                                                yakui::Color::CLEAR
                                            },
                                            None,
                                            t.radius * 0.8,
                                            0.,
                                            || {
                                                yakui::center(|| {
                                                    let color = if selected {
                                                        t.primary_foreground
                                                    } else if disabled || date.month != state.month
                                                    {
                                                        t.muted_foreground
                                                    } else {
                                                        t.foreground
                                                    };
                                                    yakui::widgets::RenderText::with_style(
                                                        date.day.to_string(),
                                                        t.text_style(14., color, 400),
                                                    )
                                                    .show();
                                                });
                                            },
                                        )
                                    });
                                    dom.end_widget::<DayWidget>(id);
                                }
                            });
                        }
                    })
                })
            });
            previous != state.selected
        },
    )
}
/// Date-picker trigger. Render `date_picker_content` at the application root.
#[track_caller]
pub fn date_picker(state: &mut CalendarState, popup: &mut crate::OverlayState) {
    let response = Button::new(
        state
            .selected_date()
            .map_or_else(|| "Pick a date".into(), Date::iso),
    )
    .variant(Variant::Outline)
    .icon(Icon::ChevronDown)
    .show();
    popup.trigger = response.id;
    if response.clicked {
        popup.toggle_from(response);
    }
}
#[track_caller]
pub fn date_picker_content(state: &mut CalendarState, popup: &mut crate::OverlayState) {
    crate::popover(popup, 228., |overlay| {
        if calendar(state) && state.mode == CalendarMode::Single {
            overlay.close();
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gregorian_boundaries() {
        assert!(Date::new(2000, 2, 29).is_some());
        assert!(Date::new(1900, 2, 29).is_none());
        assert_eq!(Date::new(1970, 1, 1).unwrap().weekday(), 4);
        assert_eq!(
            Date::new(2026, 12, 31)
                .unwrap()
                .checked_add_days(1)
                .unwrap()
                .iso(),
            "2027-01-01"
        );
    }
    #[test]
    fn date_roundtrips_across_centuries() {
        for year in [1, 1582, 1900, 1970, 2000, 2026, 9999] {
            for month in 1..=12 {
                for day in 1..=month_days(year, month) {
                    let date = Date::new(year, month, day).unwrap();
                    assert_eq!(Date::from_days(date.days()), Some(date));
                }
            }
        }
    }
    #[test]
    fn selections_are_bounded_and_ranges_normalised() {
        let date = Date::new(2026, 10, 9).unwrap();
        let mut s = CalendarState::new(date);
        s.mode = CalendarMode::Range;
        s.select(date);
        s.select(date.checked_add_days(-3).unwrap());
        assert_eq!(s.selected[1], date);
        s.minimum = Some(date);
        s.select(date.checked_add_days(-10).unwrap());
        assert_eq!(s.selected.len(), 2);
    }
}
