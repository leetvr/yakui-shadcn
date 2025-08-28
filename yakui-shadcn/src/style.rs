use yakui::{cosmic_text::FamilyOwned, style::TextStyle};

use crate::colours;
pub const PRIMARY_FONT_SIZE: f32 = 14.0;

pub fn text_label() -> TextStyle {
    let mut text_style = TextStyle::label();
    text_style.attrs.family_owned = FamilyOwned::Name("Geist".into());
    text_style.font_size = PRIMARY_FONT_SIZE;
    text_style.color = colours::PRIMARY_FOREGROUND;
    text_style
}
