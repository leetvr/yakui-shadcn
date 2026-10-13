//! An unofficial Rust port of [shadcn/ui](https://ui.shadcn.com/) to Yakui.
//! Component design and reference examples belong to shadcn/ui and its
//! contributors; interaction behavior follows [Base UI](https://base-ui.com/).
//! This crate implements that design in Rust, rather than claiming a new design system.
//!
//! Call [`add_fonts`] once, then declare components between `Yakui::start` and
//! `Yakui::finish`. Business state remains owned by the calling application.
//! The library emits Yakui paint commands and does not depend on a renderer.
mod advanced_layout;
pub mod animation;
mod avatar;
mod button_group;
mod input_group;
pub use button_group::*;
pub use input_group::*;
mod basic;
pub use avatar::*;
mod checkbox;
mod control_field;
mod switch;
pub use checkbox::*;
pub use switch::*;
mod carousel;
mod sidebar;
pub use carousel::*;
pub use sidebar::*;
mod message_scroller;
mod questionnaire;
mod toast;
pub use message_scroller::*;
pub use questionnaire::*;
pub use toast::*;
mod message;
pub use message::*;
mod otp;
pub use advanced_layout::*;
pub use otp::*;
mod calendar;
mod slider;
pub use slider::*;
mod chart;
pub use chart::*;
mod menu;
mod select;
pub use menu::*;
pub use select::*;
mod table;
mod table_slots;
pub use calendar::*;
pub use table::*;
pub use table_slots::*;
mod icons;
mod interaction;
mod layout;
mod lucide_icons;
mod navigation;
mod navigation_menu;
pub use navigation_menu::*;
mod overlay;
mod tooltip;
pub use tooltip::*;
mod theme;
pub use navigation::*;
mod editor;
mod password;
pub use basic::*;
pub use icons::*;
pub use interaction::{ControlResponse, take_pointer_drag_regions, take_pointer_drag_request};
pub use layout::*;
pub use overlay::*;
pub use theme::*;
pub use yakui::{Color, Vec2};

/// Load bundled OFL Geist and Noto symbol fallback into this instance.
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
        include_bytes!("../fonts/NotoSansSymbols2-Regular.ttf").as_slice(),
    ] {
        fonts.load_font_source(Source::Binary(Arc::new(bytes)));
    }
}
