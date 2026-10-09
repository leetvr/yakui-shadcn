//! One native editor owns all OTP slots, including paste and selection replacement.
use yakui::{
    Color, Constraints, Vec2,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    widget::{EventContext, FocusPolicy, LayoutContext, Widget},
};
#[derive(Debug, Default, Clone, Copy)]
pub enum OtpPattern {
    #[default]
    Digits,
    Alphanumeric,
    Any,
}
#[derive(Debug, Clone, Copy)]
pub struct OtpOptions {
    pub length: usize,
    pub split_after: Option<usize>,
    pub pattern: OtpPattern,
    pub disabled: bool,
    pub invalid: bool,
}
impl Default for OtpOptions {
    fn default() -> Self {
        Self {
            length: 6,
            split_after: Some(3),
            pattern: OtpPattern::Digits,
            disabled: false,
            invalid: false,
        }
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct OtpResponse {
    pub changed: bool,
    pub complete: bool,
    pub focused: bool,
}
#[derive(Debug, Clone)]
struct OtpProps {
    value: String,
    options: OtpOptions,
}
#[derive(Debug)]
struct OtpWidget {
    props: OtpProps,
    cursor: usize,
    selected_all: bool,
    focused: bool,
    pending: Option<String>,
    dismiss: Option<std::rc::Rc<crate::overlay::DismissContext>>,
}
impl OtpWidget {
    fn allowed(&self, c: char) -> bool {
        match self.props.options.pattern {
            OtpPattern::Digits => c.is_ascii_digit(),
            OtpPattern::Alphanumeric => c.is_ascii_alphanumeric(),
            OtpPattern::Any => !c.is_control(),
        }
    }
    fn insert(&mut self, text: &str) {
        let mut chars: Vec<_> = if self.selected_all {
            self.cursor = 0;
            Vec::new()
        } else {
            self.props.value.chars().collect()
        };
        self.selected_all = false;
        let incoming: Vec<_> = text.chars().filter(|c| self.allowed(*c)).collect();
        for c in incoming {
            if self.cursor >= self.props.options.length {
                break;
            }
            if self.cursor < chars.len() {
                chars[self.cursor] = c;
            } else {
                chars.push(c);
            }
            self.cursor += 1;
        }
        self.props.value = chars.into_iter().collect();
        self.pending = Some(self.props.value.clone());
    }
}
impl Widget for OtpWidget {
    type Props<'a> = OtpProps;
    type Response = (Option<String>, bool);
    fn new() -> Self {
        Self {
            props: OtpProps {
                value: String::new(),
                options: OtpOptions::default(),
            },
            cursor: 0,
            selected_all: false,
            focused: false,
            pending: None,
            dismiss: None,
        }
    }
    fn update(&mut self, mut props: OtpProps) -> Self::Response {
        let pending = self.pending.take();
        if let Some(value) = &pending {
            props.value = value.clone();
        }
        self.props = props;
        self.cursor = self
            .cursor
            .min(self.props.options.length)
            .min(self.props.value.chars().count());
        self.dismiss = yakui::util::read_scope();
        let t = crate::theme();
        let chars: Vec<_> = self.props.value.chars().collect();
        crate::row(0., || {
            for index in 0..self.props.options.length {
                if self.props.options.split_after == Some(index) && index > 0 {
                    crate::size(Vec2::new(32., 32.), || {
                        yakui::center(|| crate::text("−"));
                    });
                }
                let active = self.focused
                    && self.cursor.min(self.props.options.length.saturating_sub(1)) == index;
                let border = if self.props.options.invalid {
                    t.destructive
                } else if active {
                    t.ring
                } else {
                    t.input
                };
                crate::size(Vec2::splat(32.), || {
                    crate::surface(
                        if t.dark {
                            t.input.with_alpha(0.045)
                        } else {
                            Color::CLEAR
                        },
                        Some(border),
                        if index == 0 || index + 1 == self.props.options.length {
                            t.radius
                        } else {
                            0.
                        },
                        0.,
                        || {
                            yakui::center(|| {
                                if let Some(c) = chars.get(index) {
                                    crate::text(c.to_string());
                                } else if active {
                                    crate::size(Vec2::new(1., 16.), || {
                                        yakui::widgets::RoundRect::new(0.)
                                            .color(t.foreground)
                                            .show();
                                    });
                                }
                            });
                        },
                    )
                });
            }
        });
        (pending, self.focused)
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        if self.focused && !self.props.options.disabled {
            ctx.input.enable_text_input();
        }
        self.default_layout(ctx, constraints)
    }
    fn event_interest(&self) -> EventInterest {
        if self.props.options.disabled {
            EventInterest::empty()
        } else {
            EventInterest::MOUSE_INSIDE | EventInterest::FOCUSED_KEYBOARD
        }
    }
    fn focus_policy(&self) -> FocusPolicy {
        if self.props.options.disabled {
            FocusPolicy::empty()
        } else {
            FocusPolicy::SEQUENTIAL | FocusPolicy::POINTER
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if self.props.options.disabled {
            return EventResponse::Bubble;
        }
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            return EventResponse::Sink;
        }
        match event {
            WidgetEvent::FocusChanged { focused, .. } => {
                self.focused = *focused;
                self.selected_all = false;
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                position,
                ..
            } => {
                let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
                let mut x = position.x - rect.pos().x;
                if let Some(split) = self.props.options.split_after {
                    if x >= split as f32 * 32. {
                        x -= 32.;
                    }
                }
                self.cursor = (x / 32.).max(0.) as usize;
                self.cursor = self.cursor.min(self.props.value.chars().count());
                self.selected_all = false;
                ctx.input.set_focus(Some(ctx.dom.current()), false);
                return EventResponse::Sink;
            }
            WidgetEvent::TextInput(c, modifiers) if !modifiers.ctrl() && !modifiers.meta() => {
                self.insert(&c.to_string());
                return EventResponse::Sink;
            }
            WidgetEvent::KeyChanged {
                key,
                down: true,
                modifiers,
            } => {
                let main = if cfg!(target_os = "macos") {
                    modifiers.meta()
                } else {
                    modifiers.ctrl()
                };
                match key {
                    KeyCode::KeyA if main => self.selected_all = true,
                    KeyCode::KeyC if main => ctx
                        .dom
                        .get_global_or_init(yakui_widgets::clipboard::ClipboardHolder::default)
                        .copy(&self.props.value),
                    KeyCode::KeyV if main => {
                        if let Some(text) = ctx
                            .dom
                            .get_global_or_init(yakui_widgets::clipboard::ClipboardHolder::default)
                            .paste()
                        {
                            self.insert(&text);
                        }
                    }
                    KeyCode::ArrowLeft => self.cursor = self.cursor.saturating_sub(1),
                    KeyCode::ArrowRight => {
                        self.cursor = (self.cursor + 1).min(self.props.value.chars().count())
                    }
                    KeyCode::Home => self.cursor = 0,
                    KeyCode::End => self.cursor = self.props.value.chars().count(),
                    KeyCode::Backspace | KeyCode::Delete => {
                        let mut chars: Vec<_> = self.props.value.chars().collect();
                        if self.selected_all {
                            chars.clear();
                            self.cursor = 0;
                            self.selected_all = false;
                        } else if *key == KeyCode::Backspace && self.cursor > 0 {
                            self.cursor -= 1;
                            chars.remove(self.cursor);
                        } else if *key == KeyCode::Delete && self.cursor < chars.len() {
                            chars.remove(self.cursor);
                        }
                        self.props.value = chars.into_iter().collect();
                        self.pending = Some(self.props.value.clone());
                    }
                    _ => return EventResponse::Bubble,
                }
                return EventResponse::Sink;
            }
            _ => {}
        }
        EventResponse::Bubble
    }
}
/// A single focusable OTP editor with native numeric filtering and clipboard paste.
#[track_caller]
pub fn input_otp(value: &mut String, options: OtpOptions) -> OtpResponse {
    let response = yakui::util::widget::<OtpWidget>(OtpProps {
        value: value.clone(),
        options,
    })
    .into_inner();
    let mut changed = false;
    if let Some(next) = response.0 {
        changed = *value != next;
        *value = next;
    }
    OtpResponse {
        changed,
        complete: changed && value.chars().count() == options.length,
        focused: response.1,
    }
}
