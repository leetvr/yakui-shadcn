//! Native window and deterministic headless captures share exactly the same example tree.
mod assets;
mod avatars;
mod button_groups;
mod dropdown;
mod examples;
mod input_groups;
mod showcases;
use serde::Deserialize;
use std::{error::Error, path::PathBuf};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
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
    isolated: bool,
    state: String,
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
            isolated: false,
            state: "default".into(),
            keys: Vec::new(),
        }
    }
}
fn parse() -> Result<Option<Options>, Box<dyn Error>> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" {
            println!(
                "gallery [--component NAME] [--variant NAME] [--theme light|dark] [--state default|hover|focus|disabled|selected|open|image-error] [--keys Tab,ArrowDown,Enter] [--width 1200] [--height 800] [--isolated] [--screenshot PATH]\ngallery --list\ngallery --showcase dashboard|settings|checkout [--theme dark] [--screenshot PATH]"
            );
            return Ok(None);
        }
        if arg == "--list" {
            for component in examples::AVAILABLE {
                println!("{component}: {}", examples::variants(component).join(", "));
            }
            return Ok(None);
        }
        if arg == "--isolated" {
            options.isolated = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("Missing value for {arg}"))?;
        match arg.as_str() {
            "--component" => options.component = value,
            "--showcase" => options.showcase = Some(value),
            "--variant" => options.variant = value,
            "--screenshot" => options.screenshot = Some(value.into()),
            "--width" => options.width = value.parse()?,
            "--height" => options.height = value.parse()?,
            "--theme" => {
                options.dark = match value.as_str() {
                    "light" => false,
                    "dark" => true,
                    _ => return Err("Theme must be light or dark".into()),
                }
            }
            "--state" => options.state = value,
            "--keys" => {
                options.keys = value
                    .split(',')
                    .map(capture_key)
                    .collect::<Result<_, _>>()?
            }
            _ => return Err(format!("Unknown option {arg}").into()),
        }
    }
    if !options.keys.is_empty() && options.screenshot.is_none() {
        return Err("--keys requires --screenshot".into());
    }
    if options
        .showcase
        .as_deref()
        .is_some_and(|name| !showcases::NAMES.contains(&name))
    {
        return Err("Unknown showcase; use dashboard, settings or checkout".into());
    }
    if !examples::AVAILABLE.contains(&options.component.as_str()) {
        return Err(format!(
            "{} is not implemented yet; see reference/components.json",
            options.component
        )
        .into());
    }
    // Treat a full upstream preview name and its short variant as the same route.
    if let Some(short) = options
        .variant
        .strip_prefix(&format!("{}-", options.component))
    {
        options.variant = short.to_owned();
    }
    if !examples::variants(&options.component).contains(&options.variant.as_str()) {
        return Err(format!(
            "Unknown/unported variant {} for {}",
            options.variant, options.component
        )
        .into());
    }
    if ![
        "default",
        "hover",
        "focus",
        "disabled",
        "selected",
        "open",
        "image-error",
    ]
    .contains(&options.state.as_str())
    {
        return Err("Unsupported capture state".into());
    }
    if options.width == 0 || options.height == 0 {
        return Err("Capture dimensions must be nonzero".into());
    }
    Ok(Some(options))
}
struct Gallery {
    options: Options,
    catalogue: Catalogue,
    state: examples::ExampleState,
    showcase_state: showcases::State,
    started: std::time::Instant,
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
            started: std::time::Instant::now(),
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
            0.
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
                                                        self.state.count
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
fn capture_key(name: &str) -> Result<yakui::input::KeyCode, Box<dyn Error>> {
    use yakui::input::KeyCode::*;
    Ok(match name {
        "Tab" => Tab,
        "ArrowDown" => ArrowDown,
        "ArrowUp" => ArrowUp,
        "ArrowLeft" => ArrowLeft,
        "ArrowRight" => ArrowRight,
        "Home" => Home,
        "End" => End,
        "Enter" => Enter,
        "Space" => Space,
        "Escape" => Escape,
        "Backspace" => Backspace,
        "Delete" => Delete,
        _ => return Err(format!("Unsupported capture key {name}").into()),
    })
}
fn capture(mut gallery: Gallery, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut yak = new_yak();
    yak.set_surface_size(gallery.viewport);
    yak.set_unscaled_viewport(yakui::Rect::from_pos_size(ui::Vec2::ZERO, gallery.viewport));
    yak.set_scale_factor(1.0);
    for _ in 0..3 {
        yak.start();
        gallery.render();
        yak.finish();
    }
    if matches!(gallery.options.state.as_str(), "hover" | "focus" | "open") {
        fn first_control(yak: &yakui::Yakui, id: yakui::WidgetId) -> Option<yakui::WidgetId> {
            let node = yak.dom().get(id)?;
            if node.widget.type_name().ends_with("::ContextTarget") {
                return Some(id);
            }
            if node
                .widget
                .focus_policy()
                .contains(yakui::widget::FocusPolicy::SEQUENTIAL)
            {
                return Some(id);
            }
            node.children.iter().find_map(|id| first_control(yak, *id))
        }
        let id =
            if gallery.options.component == "button-group" && gallery.options.state == "open" {
                gallery.state.trigger.id
            } else {
                first_control(&yak, yak.dom().root())
            }
            .ok_or("This example has no focusable control")?;
        if gallery.options.state == "hover" || gallery.options.state == "open" {
            let rect = yak.layout_dom().get(id).unwrap().rect;
            yak.handle_event(yakui::event::Event::CursorMoved(Some(rect.center())));
            if gallery.options.state == "open" {
                let button = if gallery.options.component == "context-menu" {
                    yakui::input::MouseButton::Two
                } else {
                    yakui::input::MouseButton::One
                };
                for down in [true, false] {
                    yak.handle_event(yakui::event::Event::MouseButtonChanged { button, down });
                }
            }
        } else {
            yak.handle_event(yakui::event::Event::KeyChanged {
                key: yakui::input::KeyCode::Tab,
                down: true,
                modifiers: None,
            });
        }
        for _ in 0..3 {
            yak.start();
            gallery.render();
            yak.finish();
        }
    }
    if gallery.options.state == "open" && !gallery.state.is_open(&gallery.options.component) {
        return Err("The actual trigger interaction did not open this overlay".into());
    }
    for key in gallery.options.keys.clone() {
        for down in [true, false] {
            yak.handle_event(yakui::event::Event::KeyChanged {
                key,
                down,
                modifiers: None,
            });
        }
        // Consume each real event before the next key: submenu creation and
        // deferred focus must finish their frame before navigation continues.
        for _ in 0..3 {
            yak.start();
            gallery.render();
            yak.finish();
        }
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    // Save explicitly so a bare filename works as well as an absolute path.
    yakui_to_image::paint(&mut yak).save(&path)?;
    println!("{}", path.display());
    Ok(())
}
struct App {
    gallery: Gallery,
    yak: yakui::Yakui,
    window: Option<Window>,
    graphics: Option<yakui_app::Graphics>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("Adequate UI — shadcn for Yakui")
            .with_inner_size(LogicalSize::new(
                self.gallery.options.width,
                self.gallery.options.height,
            ));
        let window = event_loop.create_window(attrs).unwrap();
        window.set_ime_allowed(true);
        let graphics = pollster::block_on(yakui_app::Graphics::new(&window, 4));
        self.graphics = Some(graphics);
        self.window = Some(window);
        self.window.as_ref().unwrap().request_redraw();
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let graphics = self.graphics.as_mut().unwrap();
        graphics.handle_window_event(&mut self.yak, &event, event_loop);
        match event {
            WindowEvent::RedrawRequested => {
                self.gallery.viewport = self.yak.surface_size() / self.yak.scale_factor();
                self.yak.start();
                self.gallery.render();
                self.yak.finish();
                graphics.is_init = false;
                let c = (if self.gallery.options.dark {
                    ui::Theme::dark()
                } else {
                    ui::Theme::light()
                })
                .background
                .to_linear();
                graphics.paint(
                    &mut self.yak,
                    wgpu::Color {
                        r: c.x as f64,
                        g: c.y as f64,
                        b: c.z as f64,
                        a: 1.,
                    },
                );
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => graphics.resize(size),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let Some(options) = parse()? else {
        return Ok(());
    };
    let screenshot = options.screenshot.clone();
    let gallery = Gallery::new(options);
    if let Some(path) = screenshot {
        return capture(gallery, path);
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut App {
        gallery,
        yak: new_yak(),
        window: None,
        graphics: None,
    })?;
    Ok(())
}
