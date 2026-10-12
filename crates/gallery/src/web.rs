//! Canvas-only WebGPU host, following OATG's continuous browser frame loop.
use super::{Gallery, Options, examples, new_yak, showcases, ui};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{HtmlCanvasElement, KeyboardEvent, PointerEvent, WheelEvent};
use yakui::{
    Rect, Vec2,
    event::Event,
    input::{KeyCode, Modifiers, MouseButton},
};

struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    msaa: wgpu::TextureView,
    yak: yakui_wgpu::YakuiWgpu,
    buffers: yakui_wgpu::Buffers,
}
impl Renderer {
    async fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut config = surface
            .get_default_config(&adapter, canvas.width(), canvas.height())
            .ok_or_else(|| JsValue::from_str("WebGPU cannot present to this canvas"))?;
        // Browser canvases use UNORM. As in OATG, render through an sRGB view so
        // linear Yakui colors receive the same transfer function as native WGPU.
        let format = config.format.add_srgb_suffix();
        config.view_formats = vec![format];
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let msaa = Self::multisample(&device, format, config.width, config.height);
        let yak = yakui_wgpu::YakuiWgpu::new(device.clone(), queue.clone());
        let buffers = yak.buffers();
        Ok(Self {
            surface,
            config,
            format,
            device,
            queue,
            msaa,
            yak,
            buffers,
        })
    }
    fn multisample(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Gallery 4x MSAA"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }
    fn resize(&mut self, width: u32, height: u32) {
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.msaa = Self::multisample(&self.device, self.format, width, height);
    }
    fn paint(&mut self, yak: &mut yakui::Yakui, dark: bool) {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            _ => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.format),
            ..Default::default()
        });
        let c = if dark {
            ui::Theme::dark()
        } else {
            ui::Theme::light()
        }
        .background
        .to_linear();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa,
                    depth_slice: None,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: c.x as f64,
                            g: c.y as f64,
                            b: c.z as f64,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        let paint = self.yak.paint(
            yak,
            &mut self.buffers,
            yakui_wgpu::SurfaceInfo {
                format: self.format,
                sample_count: 4,
                color_attachment: &self.msaa,
                resolve_target: Some(&view),
            },
        );
        self.queue.submit([encoder.finish(), paint]);
        self.queue.present(output);
    }
}

struct App {
    canvas: HtmlCanvasElement,
    renderer: Renderer,
    yak: yakui::Yakui,
    gallery: Gallery,
    keys: HashSet<KeyCode>,
}
impl App {
    fn frame(&mut self) {
        resize_canvas(&self.canvas);
        let scale = web_sys::window().unwrap().device_pixel_ratio() as f32;
        let size = Vec2::new(self.canvas.width() as f32, self.canvas.height() as f32);
        self.renderer
            .resize(self.canvas.width(), self.canvas.height());
        self.yak.set_surface_size(size);
        self.yak
            .set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
        self.yak.set_scale_factor(scale);
        self.gallery.viewport = size / scale;
        self.yak.start();
        self.gallery.render();
        self.yak.finish();
        self.renderer
            .paint(&mut self.yak, self.gallery.options.dark);
    }
    fn pointer_position(&self, event: &PointerEvent) -> Vec2 {
        let rect = self.canvas.get_bounding_client_rect();
        let scale = web_sys::window().unwrap().device_pixel_ratio() as f32;
        Vec2::new(
            (event.client_x() as f64 - rect.left()) as f32,
            (event.client_y() as f64 - rect.top()) as f32,
        ) * scale
    }
    fn key(&mut self, event: KeyboardEvent, down: bool) {
        let mut modifiers = Modifiers::empty();
        modifiers.set(Modifiers::SHIFT, event.shift_key());
        modifiers.set(Modifiers::CONTROL, event.ctrl_key());
        modifiers.set(Modifiers::ALT, event.alt_key());
        modifiers.set(Modifiers::META, event.meta_key());
        self.yak.handle_event(Event::ModifiersChanged(modifiers));
        let mut handled = false;
        if let Ok(key) = event.code().parse::<KeyCode>() {
            if down {
                self.keys.insert(key);
            } else {
                self.keys.remove(&key);
            }
            handled = self.yak.handle_event(Event::KeyChanged {
                key,
                down,
                modifiers: Some(modifiers),
            });
        }
        // Canvas keyboard input follows the native host: physical keys drive
        // navigation and editing commands; printable characters drive text input.
        if down
            && !event.ctrl_key()
            && !event.meta_key()
            && !event.alt_key()
            && !event.is_composing()
        {
            let key = event.key();
            if key.chars().count() == 1 {
                handled |= self
                    .yak
                    .handle_event(Event::TextInput(key.chars().next().unwrap()));
            }
        }
        if handled || event.key() == "Tab" {
            event.prevent_default();
        }
    }
}
fn resize_canvas(canvas: &HtmlCanvasElement) {
    let scale = web_sys::window().unwrap().device_pixel_ratio();
    let width = (canvas.client_width().max(1) as f64 * scale).round() as u32;
    let height = (canvas.client_height().max(1) as f64 * scale).round() as u32;
    if canvas.width() != width {
        canvas.set_width(width);
    }
    if canvas.height() != height {
        canvas.set_height(height);
    }
}
fn button(button: i16) -> Option<MouseButton> {
    match button {
        0 => Some(MouseButton::One),
        1 => Some(MouseButton::Three),
        2 => Some(MouseButton::Two),
        _ => None,
    }
}
fn listen<E: JsCast + 'static>(
    canvas: &HtmlCanvasElement,
    name: &str,
    mut handler: impl FnMut(E) + 'static,
) -> Result<(), JsValue> {
    let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        handler(event.unchecked_into())
    });
    canvas.add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())?;
    callback.forget();
    Ok(())
}
fn install_input(app: &Rc<RefCell<App>>) -> Result<(), JsValue> {
    let canvas = app.borrow().canvas.clone();
    let state = app.clone();
    listen(&canvas, "pointermove", move |event: PointerEvent| {
        let mut app = state.borrow_mut();
        let position = app.pointer_position(&event);
        app.yak.handle_event(Event::CursorMoved(Some(position)));
    })?;
    let state = app.clone();
    listen(&canvas, "pointerdown", move |event: PointerEvent| {
        let mut app = state.borrow_mut();
        let _ = app.canvas.focus();
        let _ = app.canvas.set_pointer_capture(event.pointer_id());
        let position = app.pointer_position(&event);
        app.yak.handle_event(Event::CursorMoved(Some(position)));
        if let Some(button) = button(event.button()) {
            app.yak
                .handle_event(Event::MouseButtonChanged { button, down: true });
        }
        event.prevent_default();
    })?;
    let state = app.clone();
    listen(&canvas, "pointerup", move |event: PointerEvent| {
        let mut app = state.borrow_mut();
        let position = app.pointer_position(&event);
        app.yak.handle_event(Event::CursorMoved(Some(position)));
        if let Some(button) = button(event.button()) {
            app.yak.handle_event(Event::MouseButtonChanged {
                button,
                down: false,
            });
        }
        let _ = app.canvas.release_pointer_capture(event.pointer_id());
    })?;
    let state = app.clone();
    listen(&canvas, "pointerleave", move |_: PointerEvent| {
        state
            .borrow_mut()
            .yak
            .handle_event(Event::CursorMoved(None));
    })?;
    let state = app.clone();
    listen(&canvas, "pointercancel", move |_: PointerEvent| {
        let mut app = state.borrow_mut();
        // Cancel hover before releasing so an interrupted drag cannot click.
        app.yak.handle_event(Event::CursorMoved(None));
        for button in [MouseButton::One, MouseButton::Two, MouseButton::Three] {
            app.yak.handle_event(Event::MouseButtonChanged {
                button,
                down: false,
            });
        }
    })?;
    let state = app.clone();
    listen(&canvas, "wheel", move |event: WheelEvent| {
        let mut app = state.borrow_mut();
        let unit = match event.delta_mode() {
            1 => 25.,
            2 => app.canvas.client_height() as f64,
            _ => 1.,
        };
        let scale = web_sys::window().unwrap().device_pixel_ratio() as f32;
        app.yak.handle_event(Event::MouseScroll {
            delta: Vec2::new(
                (event.delta_x() * unit) as f32,
                (event.delta_y() * unit) as f32,
            ) * scale,
        });
        event.prevent_default();
    })?;
    listen(&canvas, "contextmenu", |event: web_sys::Event| {
        event.prevent_default()
    })?;
    for (name, down) in [("keydown", true), ("keyup", false)] {
        let state = app.clone();
        listen(&canvas, name, move |event: KeyboardEvent| {
            state.borrow_mut().key(event, down)
        })?;
    }
    let state = app.clone();
    listen(&canvas, "blur", move |_: web_sys::Event| {
        let mut app = state.borrow_mut();
        for key in std::mem::take(&mut app.keys) {
            app.yak.handle_event(Event::KeyChanged {
                key,
                down: false,
                modifiers: Some(Modifiers::empty()),
            });
        }
        app.yak.handle_event(Event::CursorMoved(None));
        for button in [MouseButton::One, MouseButton::Two, MouseButton::Three] {
            app.yak.handle_event(Event::MouseButtonChanged {
                button,
                down: false,
            });
        }
        app.yak
            .handle_event(Event::ModifiersChanged(Modifiers::empty()));
    })?;
    Ok(())
}

fn options() -> Result<Options, JsValue> {
    let params =
        web_sys::UrlSearchParams::new_with_str(&web_sys::window().unwrap().location().search()?)?;
    let mut options = Options::default();
    if let Some(name) = params.get("component") {
        if !examples::AVAILABLE.contains(&name.as_str()) {
            return Err(JsValue::from_str("Unknown component"));
        }
        options.component = name;
    }
    if let Some(variant) = params.get("variant") {
        if !examples::variants(&options.component).contains(&variant.as_str()) {
            return Err(JsValue::from_str("Unknown variant"));
        }
        options.variant = variant;
    }
    if let Some(name) = params.get("showcase") {
        if !showcases::NAMES.contains(&name.as_str()) {
            return Err(JsValue::from_str("Unknown showcase"));
        }
        options.showcase = Some(name);
    }
    options.dark = params.get("theme").as_deref() == Some("dark");
    options.isolated = params.has("isolated");
    Ok(options)
}
#[wasm_bindgen]
pub async fn start(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    resize_canvas(&canvas);
    let renderer = Renderer::new(canvas.clone()).await?;
    let app = Rc::new(RefCell::new(App {
        canvas,
        renderer,
        yak: new_yak(),
        gallery: Gallery::new(options()?),
        keys: HashSet::new(),
    }));
    install_input(&app)?;
    app.borrow_mut().frame();
    // The callback and application live for this page. Browser vsync owns every
    // frame, just as presentation drives the native host; there is no idle timer.
    let callback = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let next = callback.clone();
    *callback.borrow_mut() = Some(Closure::new(move || {
        app.borrow_mut().frame();
        web_sys::window()
            .unwrap()
            .request_animation_frame(next.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }));
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    Ok(())
}
