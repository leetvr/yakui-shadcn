use std::borrow::Cow;
use yakui::{
    MainAxisAlignment, colors,
    util::{auto_builders, widget},
    widgets::{ButtonResponse, DynamicButtonStyle, List, Pad, RenderText, RoundRect},
};
use yakui_core::{
    Alignment, Response,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::MouseButton,
    widget::{EventContext, Widget},
};

/**
A WIIIIDE button containing some text and an icon.

Responds with [ButtonResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
if yakui::button("Hello").clicked {
    println!("The button was clicked");
}
```
*/

#[track_caller]
pub fn wide_button<S: Into<Cow<'static, str>>>(text: S, icon: S) -> Response<ButtonResponse> {
    WideButton::styled(text, icon).show()
}

#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct WideButton {
    pub text: Cow<'static, str>,
    pub icon: Cow<'static, str>,
    pub alignment: Alignment,
    pub padding: Pad,
    pub border_radius: f32,
    pub style: DynamicButtonStyle,
    pub hover_style: DynamicButtonStyle,
    pub down_style: DynamicButtonStyle,
}

auto_builders!(WideButton {
    text: Cow<'static, str>,
    alignment: Alignment,
    padding: Pad,
    border_radius: f32,
    style: DynamicButtonStyle,
    hover_style: DynamicButtonStyle,
    down_style: DynamicButtonStyle,
});

impl WideButton {
    pub fn unstyled(
        text: impl Into<Cow<'static, str>>,
        icon: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            text: text.into(),
            icon: icon.into(),
            alignment: Alignment::CENTER,
            padding: Pad::ZERO,
            border_radius: 0.0,
            style: DynamicButtonStyle::default(),
            hover_style: DynamicButtonStyle::default(),
            down_style: DynamicButtonStyle::default(),
        }
    }

    pub fn styled(text: impl Into<Cow<'static, str>>, icon: impl Into<Cow<'static, str>>) -> Self {
        let style = DynamicButtonStyle {
            fill: colors::BACKGROUND_3,
            ..Default::default()
        };

        let hover_style = DynamicButtonStyle {
            fill: colors::BACKGROUND_3.adjust(1.2),
            ..Default::default()
        };

        let down_style = DynamicButtonStyle {
            fill: colors::BACKGROUND_3.adjust(0.8),
            ..Default::default()
        };

        Self {
            text: text.into(),
            icon: icon.into(),
            alignment: Alignment::CENTER,
            padding: Pad::balanced(20.0, 10.0),
            border_radius: 6.0,
            style,
            hover_style,
            down_style,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<ButtonResponse> {
        widget::<WideButtonWidget>(self)
    }
}

#[derive(Debug)]
pub struct WideButtonWidget {
    props: WideButton,
    hovering: bool,
    mouse_down: bool,
    clicked: bool,
}

impl Widget for WideButtonWidget {
    type Props<'a> = WideButton;
    type Response = ButtonResponse;

    fn new() -> Self {
        Self {
            props: WideButton::unstyled(Cow::Borrowed(""), Cow::Borrowed("")),
            hovering: false,
            mouse_down: false,
            clicked: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        let mut color = self.props.style.fill;
        let mut text_style = self.props.style.text.clone();

        if self.mouse_down {
            let style = &self.props.down_style;
            color = style.fill;
            text_style = style.text.clone();
        } else if self.hovering {
            let style = &self.props.hover_style;
            color = style.fill;
            text_style = style.text.clone();
        }

        let mut container = RoundRect::new(self.props.border_radius);
        container.color = color;
        container.show_children(|| {
            yakui::pad(self.props.padding, || {
                let mut row = List::row();
                row.main_axis_alignment = MainAxisAlignment::SpaceBetween;
                row.show(|| {
                    RenderText::with_style(self.props.text.clone(), text_style.clone()).show();
                    RenderText::with_style(self.props.icon.clone(), text_style).show();
                });
            });
        });

        let clicked = self.clicked;
        self.clicked = false;

        Self::Response {
            hovering: self.hovering,
            clicked,
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
                EventResponse::Sink
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if *inside {
                    if *down {
                        self.mouse_down = true;
                        EventResponse::Sink
                    } else if self.mouse_down {
                        self.mouse_down = false;
                        self.clicked = true;
                        EventResponse::Sink
                    } else {
                        EventResponse::Bubble
                    }
                } else {
                    if !*down {
                        self.mouse_down = false;
                    }

                    EventResponse::Bubble
                }
            }
            _ => EventResponse::Bubble,
        }
    }
}
