//! Neutral shadcn tokens, scoped to a Yakui subtree. No renderer or global mutable theme.
use std::rc::Rc;
use yakui::{
    Color,
    cosmic_text::{Attrs, AttrsOwned, Family, Weight},
    style::TextStyle,
    widgets::Scope,
};

/// Semantic tokens and dimensions used by every Adequate component.
#[derive(Debug, Clone)]
pub struct Theme {
    pub dark: bool,
    pub background: Color,
    pub foreground: Color,
    pub card: Color,
    pub card_foreground: Color,
    pub popover: Color,
    pub popover_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub secondary: Color,
    pub secondary_foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub accent: Color,
    pub accent_foreground: Color,
    pub destructive: Color,
    pub border: Color,
    pub input: Color,
    pub ring: Color,
    pub chart: [Color; 5],
    pub sidebar: Color,
    pub sidebar_foreground: Color,
    pub sidebar_primary: Color,
    pub sidebar_primary_foreground: Color,
    pub sidebar_accent: Color,
    pub sidebar_accent_foreground: Color,
    pub sidebar_border: Color,
    pub sidebar_ring: Color,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub spacing: f32,
    pub border_width: f32,
    pub radius: f32,
}

/// Convert the pinned OKLCH tokens to sRGB at startup, rather than approximating them.
fn oklch(l: f32, c: f32, h: f32) -> Color {
    let a = c * h.to_radians().cos();
    let b = c * h.to_radians().sin();
    let ll = (l + 0.39633778 * a + 0.21580376 * b).powi(3);
    let m = (l - 0.105561346 * a - 0.06385417 * b).powi(3);
    let s = (l - 0.08948418 * a - 1.2914855 * b).powi(3);
    Color::from_linear(yakui::Vec4::new(
        4.0767417 * ll - 3.3077116 * m + 0.23096994 * s,
        -1.268438 * ll + 2.6097574 * m - 0.34131938 * s,
        -0.0041960863 * ll - 0.7034186 * m + 1.7076147 * s,
        1.0,
    ))
}
impl Theme {
    /// Current default Neutral / Base UI / Nova light theme.
    pub fn light() -> Self {
        let grey = |l| oklch(l, 0.0, 0.0);
        Self {
            dark: false,
            background: grey(1.0),
            foreground: grey(0.145),
            card: grey(1.0),
            card_foreground: grey(0.145),
            popover: grey(1.0),
            popover_foreground: grey(0.145),
            primary: grey(0.205),
            primary_foreground: grey(0.985),
            secondary: grey(0.97),
            secondary_foreground: grey(0.205),
            muted: grey(0.97),
            muted_foreground: grey(0.556),
            accent: grey(0.97),
            accent_foreground: grey(0.205),
            destructive: oklch(0.577, 0.245, 27.325),
            border: grey(0.922),
            input: grey(0.922),
            ring: grey(0.708),
            chart: [
                grey(0.87),
                grey(0.556),
                grey(0.439),
                grey(0.371),
                grey(0.269),
            ],
            sidebar: grey(0.985),
            sidebar_foreground: grey(0.145),
            sidebar_primary: grey(0.205),
            sidebar_primary_foreground: grey(0.985),
            sidebar_accent: grey(0.97),
            sidebar_accent_foreground: grey(0.205),
            sidebar_border: grey(0.922),
            sidebar_ring: grey(0.708),
            font_family: "Geist".into(),
            font_size: 14.0,
            line_height: 20.0,
            spacing: 4.0,
            border_width: 1.0,
            radius: 10.0,
        }
    }
    /// Current default Neutral dark theme. Alpha border tokens remain translucent.
    pub fn dark() -> Self {
        let grey = |l| oklch(l, 0.0, 0.0);
        Self {
            dark: true,
            background: grey(0.145),
            foreground: grey(0.985),
            card: grey(0.205),
            card_foreground: grey(0.985),
            popover: grey(0.205),
            popover_foreground: grey(0.985),
            primary: grey(0.922),
            primary_foreground: grey(0.205),
            secondary: grey(0.269),
            secondary_foreground: grey(0.985),
            muted: grey(0.269),
            muted_foreground: grey(0.708),
            accent: grey(0.269),
            accent_foreground: grey(0.985),
            destructive: oklch(0.704, 0.191, 22.216),
            border: Color::WHITE.with_alpha(0.1),
            input: Color::WHITE.with_alpha(0.15),
            ring: grey(0.556),
            sidebar: grey(0.205),
            sidebar_foreground: grey(0.985),
            sidebar_primary: oklch(0.488, 0.243, 264.376),
            sidebar_primary_foreground: grey(0.985),
            sidebar_accent: grey(0.269),
            sidebar_accent_foreground: grey(0.985),
            sidebar_border: Color::WHITE.with_alpha(0.1),
            sidebar_ring: grey(0.556),
            ..Self::light()
        }
    }
    /// Geist typography with an explicit line height, size, colour and weight.
    pub fn text_style(&self, size: f32, color: Color, weight: u16) -> TextStyle {
        TextStyle {
            font_size: size,
            line_height_override: Some(if size == self.font_size {
                self.line_height
            } else {
                size * 1.4
            }),
            color,
            attrs: AttrsOwned::new(
                &Attrs::new()
                    .family(Family::Name(&self.font_family))
                    .weight(Weight(weight)),
            ),
            ..TextStyle::default()
        }
    }
}
impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
/// Scope tokens to a component tree. Nested scopes override only their descendants.
#[track_caller]
pub fn with_theme(theme: Theme, children: impl FnOnce()) {
    Scope::new(theme).show(children);
}
/// Read the nearest theme, falling back to the default light theme.
pub fn theme() -> Rc<Theme> {
    yakui::util::read_scope::<Theme>().unwrap_or_else(|| Rc::new(Theme::default()))
}
