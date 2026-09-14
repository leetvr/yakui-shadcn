use std::borrow::Cow;

use yakui::{
    Alignment, Response,
    widgets::{Button, ButtonResponse, DynamicButtonStyle, Pad},
};

use crate::{colours, style};

pub fn button(text: impl Into<Cow<'static, str>>) -> Response<ButtonResponse> {
    let style = DynamicButtonStyle {
        fill: colours::PRIMARY,
        text: style::text_label(),
        ..Default::default()
    };

    let hover_style = DynamicButtonStyle {
        fill: colours::PRIMARY.adjust(1.0),
        text: style::text_label(),
        ..Default::default()
    };

    let down_style = DynamicButtonStyle {
        fill: colours::PRIMARY.adjust(0.8),
        text: style::text_label(),
        ..Default::default()
    };

    Button {
        text: text.into(),
        alignment: Alignment::CENTER,
        padding: Pad::balanced(16.0, 8.0),
        border_radius: 8.0,
        style,
        hover_style,
        down_style,
    }
    .show()
}
