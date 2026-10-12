//! Desktop window and deterministic PNG capture host.
use super::*;
use std::error::Error;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
fn parse() -> Result<Option<Options>, Box<dyn Error>> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" {
            println!(
                "gallery [--component NAME] [--variant NAME] [--theme light|dark] [--state default|hover|focus|disabled|selected|open|image-error] [--keys Tab,ArrowDown,Enter] [--width 1200] [--height 800] [--scale-factor 1] [--isolated] [--screenshot PATH] [--animation-time-ms 50]\ngallery --list\ngallery --showcase dashboard|settings|checkout [--theme dark] [--screenshot PATH]"
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
            "--animation-time-ms" => options.animation_time_ms = Some(value.parse()?),
            "--width" => options.width = value.parse()?,
            "--height" => options.height = value.parse()?,
            "--scale-factor" => options.scale_factor = value.parse()?,
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
    if options.animation_time_ms.is_some() && options.screenshot.is_none() {
        return Err("--animation-time-ms requires --screenshot".into());
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
    if !options.scale_factor.is_finite() || options.scale_factor <= 0. {
        return Err("Scale factor must be finite and positive".into());
    }
    Ok(Some(options))
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
fn capture_frame(gallery: &mut Gallery, yak: &mut yakui::Yakui, milliseconds: u64) {
    let animated = gallery.options.animation_time_ms.is_some();
    yak.start();
    ui::animation::with_motion_enabled(animated, || {
        ui::animation::with_animation_time(std::time::Duration::from_millis(milliseconds), || {
            gallery.render()
        });
    });
    yak.finish();
}
fn capture(mut gallery: Gallery, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut yak = new_yak();
    let scale = gallery.options.scale_factor;
    let physical_size = (gallery.viewport * scale).round();
    yak.set_surface_size(physical_size);
    yak.set_unscaled_viewport(yakui::Rect::from_pos_size(ui::Vec2::ZERO, physical_size));
    yak.set_scale_factor(scale);
    for _ in 0..3 {
        capture_frame(&mut gallery, &mut yak, 0);
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
        let id = if matches!(
            gallery.options.component.as_str(),
            "button-group" | "select" | "tooltip"
        ) && gallery.options.state == "open"
        {
            gallery.state.trigger.id
        } else {
            first_control(&yak, yak.dom().root())
        }
        .ok_or("This example has no focusable control")?;
        if gallery.options.state == "hover" || gallery.options.state == "open" {
            let rect = yak.layout_dom().get(id).unwrap().rect;
            let mut position = rect.center();
            if gallery.options.component == "slider" {
                // Hover the first actual thumb, like the browser harness. The
                // center of a range control is not necessarily over a thumb.
                let (value, vertical) = match gallery.options.variant.as_str() {
                    "range" => (gallery.state.slider_range[0], false),
                    "vertical" => (gallery.state.slider_vertical[0], true),
                    _ => (gallery.state.slider_value, false),
                };
                if vertical {
                    position.y = rect.max().y - 6. - value as f32 / 100. * (rect.size().y - 12.);
                } else {
                    position.x = rect.pos().x + 6. + value as f32 / 100. * (rect.size().x - 12.);
                }
            }
            yak.handle_event(yakui::event::Event::CursorMoved(Some(position * scale)));
            let hover_popup = gallery.options.component == "tooltip"
                || (gallery.options.component == "button-group"
                    && matches!(gallery.options.variant.as_str(), "nested" | "input-group"));
            if gallery.options.state == "open" && !hover_popup {
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
            capture_frame(&mut gallery, &mut yak, 1000);
        }
    }
    if gallery.options.state == "open" && !gallery.state.is_open(&gallery.options.component) {
        return Err("The actual trigger interaction did not open this overlay".into());
    }
    let mut action_time = if matches!(gallery.options.state.as_str(), "hover" | "focus" | "open") {
        1000
    } else {
        0
    };
    for key in gallery.options.keys.clone() {
        capture_frame(&mut gallery, &mut yak, action_time + 1000);
        action_time += 2000;
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
            capture_frame(&mut gallery, &mut yak, action_time);
        }
    }
    let final_time = action_time + gallery.options.animation_time_ms.unwrap_or(1000);
    capture_frame(&mut gallery, &mut yak, final_time);
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
pub fn run() -> Result<(), Box<dyn Error>> {
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
