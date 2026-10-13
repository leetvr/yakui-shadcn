//! Gallery-only source display and host clipboard access.
use std::{cell::RefCell, rc::Rc};
use yakui_shadcn as ui;

pub struct Snippet {
    pub component: &'static str,
    pub variant: &'static str,
    pub example: &'static str,
    pub state: &'static str,
    pub helpers: &'static str,
}
include!(concat!(env!("OUT_DIR"), "/snippets.rs"));

#[derive(Default)]
pub struct Panel {
    open: bool,
    section: usize,
    cached: &'static str,
    lines: Vec<Vec<(String, Ink)>>,
    feedback: Rc<RefCell<&'static str>>,
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Option<arboard::Clipboard>,
}
impl Panel {
    pub fn show(&mut self, component: &str, variant: &str, width: f32) {
        let snippet = SNIPPETS
            .iter()
            .find(|s| s.component == component && s.variant == variant)
            .expect("every gallery route has extracted source");
        ui::collapsible("Code", &mut self.open, || {
            ui::column(12., || {
                ui::row(8., || {
                    for (i, label) in ["Example", "State", "Helpers"].into_iter().enumerate() {
                        if i == 2 && snippet.helpers.is_empty() {
                            continue;
                        }
                        if ui::Button::new(label)
                            .size(ui::ButtonSize::Sm)
                            .variant(if i == self.section {
                                ui::Variant::Secondary
                            } else {
                                ui::Variant::Ghost
                            })
                            .show()
                            .clicked
                        {
                            self.section = i;
                            *self.feedback.borrow_mut() = "";
                        }
                    }
                });
                let code = match self.section {
                    1 => snippet.state,
                    2 => snippet.helpers,
                    _ => snippet.example,
                };
                ui::row(8., || {
                    let feedback = *self.feedback.borrow();
                    if ui::Button::new(if feedback.is_empty() {
                        "Copy"
                    } else {
                        feedback
                    })
                    .icon(ui::Icon::Copy)
                    .size(ui::ButtonSize::Sm)
                    .variant(ui::Variant::Outline)
                    .show()
                    .clicked
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if self.clipboard.is_none() {
                                self.clipboard = arboard::Clipboard::new().ok();
                            }
                            *self.feedback.borrow_mut() = match &mut self.clipboard {
                                Some(c) => {
                                    if c.set_text(code).is_ok() {
                                        "Copied"
                                    } else {
                                        "Copy failed"
                                    }
                                }
                                _ => "Copy failed",
                            };
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            *self.feedback.borrow_mut() = "Copying…";
                            let feedback = self.feedback.clone();
                            let promise = web_sys::window()
                                .unwrap()
                                .navigator()
                                .clipboard()
                                .write_text(code);
                            wasm_bindgen_futures::spawn_local(async move {
                                *feedback.borrow_mut() = if wasm_bindgen_futures::JsFuture::from(
                                    promise,
                                )
                                .await
                                .is_ok()
                                {
                                    "Copied"
                                } else {
                                    "Copy failed"
                                };
                            });
                        }
                    }
                    super::chrome::text(
                        "Rust · from the running example",
                        12.,
                        ui::theme().muted_foreground,
                        400,
                    );
                });
                if self.cached != code {
                    self.lines = code.lines().map(highlight).collect();
                    self.cached = code;
                }
                ui::width(width, || {
                    ui::surface(ui::theme().muted, None, 6., 16., || {
                        ui::scroll_area(320., || {
                            ui::column(0., || {
                                let columns = ((width - 32.).max(120.) / 8.5) as usize;
                                for line in &self.lines {
                                    // Wrap at a character boundary so wide source never spills
                                    // into adjacent content. Monospaced cells preserve indentation.
                                    for line in wrap(line, columns) {
                                        ui::row(0., || {
                                            if line.is_empty() {
                                                ui::size(ui::Vec2::new(1., 20.), || {});
                                            }
                                            for (text, ink) in line {
                                                let mut style =
                                                    ui::theme().text_style(13., ink.color(), 400);
                                                style.attrs.family_owned =
                                                    yakui::cosmic_text::FamilyOwned::Name(
                                                        "Roboto Mono".into(),
                                                    );
                                                style.line_height_override = Some(20.);
                                                if text.trim().is_empty() {
                                                    ui::size(
                                                        ui::Vec2::new(
                                                            text.chars().count() as f32 * 8.,
                                                            20.,
                                                        ),
                                                        || {},
                                                    );
                                                } else {
                                                    // Let the shaper measure each run. Tight guessed glyph
                                                    // widths can wrap away its last character at fractional DPI.
                                                    yakui::widgets::RenderText::with_style(
                                                        text, style,
                                                    )
                                                    .show();
                                                }
                                            }
                                        });
                                    }
                                }
                            });
                        });
                    });
                });
            });
        });
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Ink {
    Plain,
    Keyword,
    String,
    Number,
    Comment,
    Type,
}
impl Ink {
    fn color(self) -> ui::Color {
        let dark = ui::theme().dark;
        ui::Color::hex(match (self, dark) {
            (Self::Keyword, false) => 0x7c3aed,
            (Self::Keyword, true) => 0xc4b5fd,
            (Self::String, false) => 0x047857,
            (Self::String, true) => 0x6ee7b7,
            (Self::Number, false) => 0xb45309,
            (Self::Number, true) => 0xfcd34d,
            (Self::Type, false) => 0x0369a1,
            (Self::Type, true) => 0x7dd3fc,
            (Self::Comment, _) => return ui::theme().muted_foreground,
            _ => return ui::theme().foreground,
        })
    }
}
fn highlight(line: &str) -> Vec<(String, Ink)> {
    let chars: Vec<_> = line.chars().collect();
    let mut tokens = vec![];
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        let ink = if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            i = chars.len();
            Ink::Comment
        } else if chars[i] == '"' {
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i = (i + 2).min(chars.len());
                } else {
                    i += 1;
                    if chars[i - 1] == '"' {
                        break;
                    }
                }
            }
            Ink::String
        } else if chars[i].is_alphanumeric() || chars[i] == '_' {
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if [
                "fn", "let", "mut", "pub", "use", "as", "if", "else", "match", "for", "in", "impl",
                "struct", "enum", "return", "move", "true", "false", "mod", "const",
            ]
            .contains(&word.as_str())
            {
                Ink::Keyword
            } else if chars[start].is_numeric() {
                Ink::Number
            } else if chars[start].is_uppercase() {
                Ink::Type
            } else {
                Ink::Plain
            }
        } else {
            i += 1;
            Ink::Plain
        };
        let text: String = chars[start..i].iter().collect();
        if let Some((previous, previous_ink)) = tokens.last_mut() {
            if *previous_ink == ink {
                String::push_str(previous, &text);
                continue;
            }
        }
        tokens.push((text, ink));
    }
    tokens
}
fn wrap(tokens: &[(String, Ink)], columns: usize) -> Vec<Vec<(String, Ink)>> {
    let mut lines = vec![vec![]];
    let mut used = 0;
    for (text, ink) in tokens {
        for chunk in text.chars().collect::<Vec<_>>().chunks(columns.max(1)) {
            let mut chars = chunk.iter().copied().peekable();
            while chars.peek().is_some() {
                if used == columns {
                    lines.push(vec![]);
                    used = 0;
                }
                let text: String = chars.by_ref().take(columns - used).collect();
                used += text.chars().count();
                lines.last_mut().unwrap().push((text, *ink));
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_live_route_has_real_source() {
        for component in crate::examples::AVAILABLE {
            for variant in crate::examples::variants(component) {
                let snippet = SNIPPETS
                    .iter()
                    .find(|s| s.component == *component && s.variant == *variant)
                    .unwrap();
                assert!(
                    snippet.example.contains("pub fn render"),
                    "{component}/{variant}"
                );
                assert!(snippet.state.contains("struct ExampleState"));
            }
        }
        let button = SNIPPETS
            .iter()
            .find(|s| s.component == "button" && s.variant == "outline")
            .unwrap();
        assert!(button.example.contains("V::Outline"));
        assert!(!button.example.contains("V::Destructive"));
        let dialog = SNIPPETS.iter().find(|s| s.component == "dialog").unwrap();
        assert!(dialog.example.contains("ui::dialog"));
        assert!(dialog.state.contains("OverlayState"));
    }
    #[test]
    fn syntax_wrapping_preserves_every_character() {
        let source = "    let title = \"héllo \\\"世界\\\"\"; // comment";
        let lines = wrap(&highlight(source), 12);
        let reconstructed: String = lines.iter().flatten().map(|(s, _)| s.as_str()).collect();
        assert_eq!(reconstructed, source);
        assert!(
            lines
                .iter()
                .all(|l| l.iter().map(|(s, _)| s.chars().count()).sum::<usize>() <= 12)
        );
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    use crate::examples::{self, ExampleState};
    use yakui::{Rect, Vec2, WidgetId, Yakui, event::Event, input::MouseButton};
    fn controls(yak: &Yakui, id: WidgetId, out: &mut Vec<WidgetId>) {
        let node = yak.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::ControlWidget") {
            out.push(id);
        }
        for child in &node.children {
            controls(yak, *child, out);
        }
    }
    fn click(yak: &mut Yakui, id: WidgetId) {
        let position = yak.layout_dom().get(id).unwrap().rect.center();
        yak.handle_event(Event::CursorMoved(Some(position)));
        for down in [true, false] {
            yak.handle_event(Event::MouseButtonChanged {
                button: MouseButton::One,
                down,
            });
        }
    }
    fn frame(yak: &mut Yakui, state: &mut ExampleState, panel: &mut Panel) -> Vec<WidgetId> {
        yak.start();
        ui::column(16., || {
            examples::render("button", "default", state, false);
            panel.show("button", "default", 640.);
        });
        yak.finish();
        let mut out = vec![];
        controls(yak, yak.dom().root(), &mut out);
        out
    }
    #[test]
    fn opening_and_closing_source_preserves_the_live_preview() {
        let mut yak = crate::new_yak();
        yak.set_surface_size(Vec2::new(1000., 1000.));
        yak.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, yak.surface_size()));
        let mut state = ExampleState::default();
        let mut panel = Panel::default();
        let initial = frame(&mut yak, &mut state, &mut panel);
        click(&mut yak, initial[0]);
        frame(&mut yak, &mut state, &mut panel);
        assert_eq!(state.count, 1);
        click(&mut yak, initial[1]);
        let expanded = frame(&mut yak, &mut state, &mut panel);
        assert!(panel.open);
        assert_eq!(expanded[0], initial[0]);
        assert_eq!(state.count, 1);
        click(&mut yak, expanded[1]);
        let closed = frame(&mut yak, &mut state, &mut panel);
        assert!(!panel.open);
        assert_eq!(closed[0], initial[0]);
        click(&mut yak, closed[0]);
        frame(&mut yak, &mut state, &mut panel);
        assert_eq!(state.count, 2);
    }
}
