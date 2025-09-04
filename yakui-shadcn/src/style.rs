use yakui::{
    cosmic_text::{FamilyOwned, Metrics, Weight},
    style::TextStyle,
};

use crate::colours;
pub const PRIMARY_FONT_SIZE: f32 = 16.0;
pub const SUBHEADER_FONT_SIZE: f32 = 12.0;

pub fn text_label() -> TextStyle {
    let mut text_style = TextStyle::label();
    text_style.attrs.family_owned = FamilyOwned::Name("Geist".into());
    text_style.font_size = PRIMARY_FONT_SIZE;
    text_style.color = colours::PRIMARY_FOREGROUND;
    text_style
}

pub fn text_body() -> TextStyle {
    let mut text_style = TextStyle::label();

    text_style.attrs.family_owned = FamilyOwned::Name("Geist".into());
    text_style.font_size = PRIMARY_FONT_SIZE;
    text_style.color = colours::PRIMARY_HEADER;
    text_style.attrs.weight = Weight::NORMAL;
    text_style
}

pub fn text_header() -> TextStyle {
    let mut text_style = TextStyle::label();

    text_style.attrs.family_owned = FamilyOwned::Name("Geist".into());
    text_style.font_size = PRIMARY_FONT_SIZE;
    text_style.color = colours::PRIMARY_HEADER;
    text_style.attrs.weight = Weight::MEDIUM;

    let scale = 2.0; // TODO
    let metrics = Metrics::new(PRIMARY_FONT_SIZE, 17.5).scale(scale);
    text_style.attrs.metrics_opt = Some(metrics.into());
    text_style
}

pub fn text_subheader() -> TextStyle {
    let mut text_style = TextStyle::label();

    text_style.attrs.family_owned = FamilyOwned::Name("Geist".into());
    text_style.font_size = SUBHEADER_FONT_SIZE;
    text_style.color = colours::SUBHEADER;
    text_style.attrs.weight = Weight::MEDIUM;

    let scale = 2.0; // TODO
    let metrics = Metrics::new(SUBHEADER_FONT_SIZE, 16.0).scale(scale);
    text_style.attrs.metrics_opt = Some(metrics.into());
    text_style
}

pub fn icon_label() -> TextStyle {
    let mut text_style = TextStyle::label();
    text_style.attrs.family_owned = FamilyOwned::Name("Lucide".into());
    text_style.font_size = PRIMARY_FONT_SIZE;
    text_style.color = colours::PRIMARY_FOREGROUND;
    text_style
}
