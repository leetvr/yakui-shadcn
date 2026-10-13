//! Shared gallery component tree for desktop and browser WGPU hosts.
mod assets;
mod avatars;
mod button_groups;
mod chrome;
mod dropdown;
mod examples;
mod input_groups;
mod navigation_menus;
mod selects;
mod showcases;
mod source;
mod tooltips;
use serde::Deserialize;
use std::path::PathBuf;
use yakui_shadcn as ui;

#[derive(Deserialize)]
struct Catalogue {
    components: Vec<Entry>,
}
#[derive(Deserialize)]
struct Entry {
    name: String,
    title: String,
    description: String,
}
#[derive(Clone)]
struct Options {
    component: String,
    showcase: Option<String>,
    variant: String,
    dark: bool,
    screenshot: Option<PathBuf>,
    width: u32,
    height: u32,
    #[cfg(not(target_arch = "wasm32"))]
    scale_factor: f32,
    isolated: bool,
    animation_time_ms: Option<u64>,
    state: String,
    #[cfg(not(target_arch = "wasm32"))]
    keys: Vec<yakui::input::KeyCode>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            component: "button".into(),
            showcase: None,
            variant: "default".into(),
            dark: false,
            screenshot: None,
            width: 1200,
            height: 800,
            #[cfg(not(target_arch = "wasm32"))]
            scale_factor: 1.,
            isolated: false,
            animation_time_ms: None,
            state: "default".into(),
            #[cfg(not(target_arch = "wasm32"))]
            keys: Vec::new(),
        }
    }
}
struct Gallery {
    options: Options,
    catalogue: Catalogue,
    state: examples::ExampleState,
    showcase_state: showcases::State,
    started: web_time::Instant,
    route: String,
    page: ui::animation::Transition<f32>,
    panels: std::collections::BTreeMap<String, source::Panel>,
    viewport: ui::Vec2,
    states: std::collections::BTreeMap<String, examples::ExampleState>,
}
impl Gallery {
    fn new(options: Options) -> Self {
        let viewport = ui::Vec2::new(options.width as f32, options.height as f32);
        let mut state = examples::ExampleState::default();
        if options.state == "image-error" {
            state.avatar_images = false;
        }
        if options.state == "selected" {
            state.checked = true;
            if options.component == "checkbox" {
                match options.variant.as_str() {
                    "description" => state.checkbox_fields[1] = false,
                    "group" => state.checkbox_group[0] = false,
                    "table" => state.table_selected.fill(true),
                    "disabled" => {}
                    _ => state.checkbox_fields[0] = true,
                }
            }
            if options.component == "switch" {
                match options.variant.as_str() {
                    "choice-card" => state.switch_values[0] = true,
                    "sizes" => state.switch_sizes[0] = true,
                    _ => {}
                }
            }
            state.toggle = true;
            state.overlay.open = true;
            state.select.popup.open = true;
            state.menu.popup.open = true;
        }
        Self {
            options,
            catalogue: serde_json::from_str(include_str!("../../../reference/components.json"))
                .unwrap(),
            state,
            showcase_state: Default::default(),
            started: web_time::Instant::now(),
            route: String::new(),
            page: ui::animation::Transition::new(1.),
            panels: Default::default(),
            viewport,
            states: Default::default(),
        }
    }
    fn render(&mut self) {
        let t = if self.options.dark {
            ui::Theme::dark()
        } else {
            ui::Theme::light()
        };
        self.state.phase = if self.options.screenshot.is_some() {
            self.options
                .animation_time_ms
                .map_or(0., |ms| ms as f32 * 0.006)
        } else {
            self.started.elapsed().as_secs_f32() * 6.
        };
        for state in self.states.values_mut() {
            state.phase = self.state.phase;
        }
        ui::with_theme(t.clone(), || {
            ui::size(self.viewport, || {
                ui::surface(t.background, None, 0., 0., || {
                    if self.options.isolated {
                        yakui::center(|| {
                            examples::render(
                                &self.options.component,
                                &self.options.variant,
                                &mut self.state,
                                self.options.state == "disabled",
                            )
                        });
                        return;
                    }
                    chrome::render(self);
                })
            });
            if let Some(name) = &self.options.showcase {
                showcases::overlays(name, &mut self.showcase_state);
            } else if self.options.isolated {
                examples::overlays(
                    &self.options.component,
                    &self.options.variant,
                    &mut self.state,
                );
            } else {
                for variant in examples::variants(&self.options.component) {
                    let state = self
                        .states
                        .entry(format!("{}:{variant}", self.options.component))
                        .or_default();
                    examples::overlays(&self.options.component, variant, state);
                }
            }
        });
    }
}
fn new_yak() -> yakui::Yakui {
    let mut yak = yakui::Yakui::new();
    ui::add_fonts(&mut yak);
    assets::load(&mut yak);
    let fonts = yak.dom().get_global_or_init(yakui::font::Fonts::default);
    fonts.load_font_source(yakui::cosmic_text::fontdb::Source::Binary(
        std::sync::Arc::new(include_bytes!("../assets/fonts/RobotoMono-Regular.ttf").as_slice()),
    ));
    yak
}

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::run as run_native;
#[cfg(target_arch = "wasm32")]
mod web;
