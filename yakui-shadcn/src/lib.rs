mod button;
pub mod colours;
pub mod icons;
mod sidebar;
mod style;
pub mod text;
pub use button::button;
pub use sidebar::{SidebarItem, sidebar};
use std::sync::Arc;
use yakui::{cosmic_text::fontdb, font::Fonts};

pub fn add_fonts(yak: &mut yakui::Yakui) {
    let fonts = yak.dom().get_global_or_init(|| Fonts::default());
    let font_data: &[&[u8]] = &[
        include_bytes!("../fonts/Geist-Bold.ttf"),
        include_bytes!("../fonts/Geist-Light.ttf"),
        include_bytes!("../fonts/Geist-Medium.ttf"),
        include_bytes!("../fonts/Geist-Regular.ttf"),
        include_bytes!("../fonts/Geist-Regular.ttf"),
        include_bytes!("../fonts/Lucide.ttf"),
    ];

    for font in font_data.into_iter() {
        fonts.load_font_source(fontdb::Source::Binary(Arc::from(font)));
    }
}
