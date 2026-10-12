//! Shared gallery component tree for desktop and browser WGPU hosts.
mod assets;
mod avatars;
mod button_groups;
mod dropdown;
mod examples;
mod input_groups;
mod navigation_menus;
mod selects;
mod showcases;
mod tooltips;
use serde::Deserialize;
use std::path::PathBuf;
use yakui_adequate_ui as ui;

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
                    if let Some(name) = self.options.showcase.clone() {
                        ui::scroll_area(self.viewport.y, || {
                            ui::surface(t.background, None, 0., 24., || {
                                ui::column(24., || {
                                    ui::row(8., || {
                                        for route in showcases::NAMES {
                                            if ui::Button::new(*route)
                                                .variant(ui::Variant::Outline)
                                                .show()
                                                .clicked
                                            {
                                                self.options.showcase = Some((*route).into());
                                            }
                                        }
                                        if ui::Button::new("Components")
                                            .variant(ui::Variant::Ghost)
                                            .show()
                                            .clicked
                                        {
                                            self.options.showcase = None;
                                        }
                                        if ui::Button::new("Toggle theme")
                                            .variant(ui::Variant::Ghost)
                                            .show()
                                            .clicked
                                        {
                                            self.options.dark = !self.options.dark;
                                        }
                                    });
                                    showcases::render(
                                        &name,
                                        &mut self.showcase_state,
                                        self.viewport.x - 48.,
                                    );
                                });
                            });
                        });
                        return;
                    }
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
                    ui::column(0., || {
                        ui::fill_width(|| {
                            ui::surface(t.background, Some(t.border), 0., 16., || {
                                ui::row(16., || {
                                    ui::label("Adequate UI");
                                    ui::muted("shadcn / Base UI / Nova");
                                    for name in showcases::NAMES {
                                        if ui::Button::new(*name)
                                            .variant(ui::Variant::Ghost)
                                            .show()
                                            .clicked
                                        {
                                            self.options.showcase = Some((*name).into());
                                        }
                                    }
                                    if ui::Button::new(if self.options.dark {
                                        "Light theme"
                                    } else {
                                        "Dark theme"
                                    })
                                    .variant(ui::Variant::Outline)
                                    .show()
                                    .clicked
                                    {
                                        self.options.dark = !self.options.dark;
                                    }
                                })
                            })
                        });
                        ui::row(0., || {
                            let navigation = self.viewport.x >= 800.;
                            if navigation {
                                ui::width(224., || {
                                    ui::surface(t.sidebar, None, 0., 16., || {
                                        ui::scroll_area(self.viewport.y - 96., || {
                                            ui::column(4., || {
                                                ui::label("Components");
                                                for entry in &self.catalogue.components {
                                                    let active =
                                                        entry.name == self.options.component;
                                                    if ui::Button::new(entry.title.clone())
                                                        .variant(if active {
                                                            ui::Variant::Secondary
                                                        } else {
                                                            ui::Variant::Ghost
                                                        })
                                                        .show()
                                                        .clicked
                                                    {
                                                        self.options.component = entry.name.clone();
                                                        self.options.variant = "default".into();
                                                        self.state =
                                                            examples::ExampleState::default();
                                                    }
                                                }
                                            })
                                        })
                                    })
                                });
                            }
                            ui::width(
                                (self.viewport.x - if navigation { 224. } else { 0. }).max(200.),
                                || {
                                    ui::scroll_area(self.viewport.y - 72., || {
                                        ui::surface(t.background, None, 0., 32., || {
                                            ui::column(24., || {
                                                if let Some(entry) = self
                                                    .catalogue
                                                    .components
                                                    .iter()
                                                    .find(|e| e.name == self.options.component)
                                                {
                                                    ui::heading(2, entry.title.clone());
                                                    ui::muted(entry.description.clone());
                                                }
                                                if !navigation {
                                                    ui::row(8., || {
                                                        for (delta, title) in [
                                                            (-1, "Previous component"),
                                                            (1, "Next component"),
                                                        ] {
                                                            if ui::Button::new(title)
                                                                .variant(ui::Variant::Outline)
                                                                .show()
                                                                .clicked
                                                            {
                                                                let pos = self
                                                                    .catalogue
                                                                    .components
                                                                    .iter()
                                                                    .position(|e| {
                                                                        e.name
                                                                            == self
                                                                                .options
                                                                                .component
                                                                    })
                                                                    .unwrap_or(0);
                                                                let pos = (pos as isize + delta)
                                                                    .rem_euclid(
                                                                        self.catalogue
                                                                            .components
                                                                            .len()
                                                                            as isize,
                                                                    )
                                                                    as usize;
                                                                self.options.component =
                                                                    self.catalogue.components[pos]
                                                                        .name
                                                                        .clone();
                                                                self.state =
                                                                    examples::ExampleState::default(
                                                                    );
                                                            }
                                                        }
                                                    });
                                                }
                                                if examples::AVAILABLE
                                                    .contains(&self.options.component.as_str())
                                                {
                                                    for variant in
                                                        examples::variants(&self.options.component)
                                                    {
                                                        ui::label(variant.to_string());
                                                        ui::fill_width(|| {
                                                            ui::surface(
                                                                t.background,
                                                                Some(t.border),
                                                                t.radius,
                                                                24.,
                                                                || {
                                                                    ui::min_size(
                                                                        ui::Vec2::new(0., 112.),
                                                                        || {
                                                                            examples::render(
                                                                                &self
                                                                                    .options
                                                                                    .component,
                                                                                variant,
                                                                                self.states
                                                                                    .entry(format!(
                                                                            "{}:{variant}",
                                                                            self.options.component
                                                                        ))
                                                                                    .or_default(),
                                                                                false,
                                                                            )
                                                                        },
                                                                    );
                                                                },
                                                            )
                                                        });
                                                    }
                                                    ui::muted(format!(
                                                        "Actions: {}",
                                                        self.states
                                                            .iter()
                                                            .filter(|(key, _)| key.starts_with(
                                                                &format!(
                                                                    "{}:",
                                                                    self.options.component
                                                                )
                                                            ))
                                                            .map(|(_, state)| state.count as u64)
                                                            .sum::<u64>()
                                                    ));
                                                } else {
                                                    examples::render(
                                                        &self.options.component,
                                                        "default",
                                                        &mut self.state,
                                                        false,
                                                    );
                                                }
                                            })
                                        });
                                    })
                                },
                            );
                        });
                    });
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
    yak
}

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::run as run_native;
#[cfg(target_arch = "wasm32")]
mod web;
