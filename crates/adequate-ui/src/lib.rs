//! Native shadcn/Base UI components built directly on Yakui.
//!
//! Call [`add_fonts`] once, then declare components between `Yakui::start` and
//! `Yakui::finish`. Business state remains owned by the calling application.
//! The library emits Yakui paint commands and does not depend on a renderer.
mod basic;
mod icons;
mod interaction;
mod layout;
mod navigation;
mod overlay;
mod theme;
pub use navigation::*;
mod editor;
pub use basic::*;
pub use icons::*;
pub use interaction::ControlResponse;
pub use layout::*;
pub use overlay::*;
pub use theme::*;
pub use yakui::{Color, Vec2};

/// Load the bundled OFL Geist family into this Yakui instance.
/// Call once per instance, before the first UI frame.
pub fn add_fonts(yak: &mut yakui::Yakui) {
    use std::sync::Arc;
    use yakui::{cosmic_text::fontdb::Source, font::Fonts};
    let fonts = yak.dom().get_global_or_init(Fonts::default);
    for bytes in [
        include_bytes!("../fonts/Geist-Regular.ttf").as_slice(),
        include_bytes!("../fonts/Geist-Medium.ttf").as_slice(),
        include_bytes!("../fonts/Geist-Bold.ttf").as_slice(),
        include_bytes!("../fonts/Geist-Light.ttf").as_slice(),
    ] {
        fonts.load_font_source(Source::Binary(Arc::new(bytes)));
    }
}
