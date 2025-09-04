use yakui::widgets::Pad;

use crate::style::{self, PRIMARY_FONT_SIZE, SUBHEADER_FONT_SIZE};

pub fn header(text: String) {
    let mut label = yakui::widgets::Text::new(PRIMARY_FONT_SIZE, text);
    label.style = style::text_header();
    label.show();
}

pub fn subheader(text: String) {
    let mut label = yakui::widgets::Text::new(SUBHEADER_FONT_SIZE, text);
    label.style = style::text_subheader();
    label.show();
}

pub fn body(text: String) {
    let mut label = yakui::widgets::Text::new(PRIMARY_FONT_SIZE, text);
    label.style = style::text_body();
    label.show();
}
