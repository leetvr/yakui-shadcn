//! WebGPU canvas and browser keyboard bridge, with a continuous vsync frame loop.
use super::{Gallery, Options, examples, new_yak, showcases, ui};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{
    HtmlCanvasElement, HtmlElement, HtmlTextAreaElement, InputEvent, KeyboardEvent, PointerEvent,
    WheelEvent,
};
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
    touch: Option<Touch>,
    keyboard: HtmlTextAreaElement,
    browser: BrowserPage,
}
struct Touch {
    id: i32,
    origin: Vec2,
    previous: Vec2,
    scrolling: bool,
    control_drag: bool,
    native_page: bool,
}
// web-sys' stable scroll_top accessor truncates to integer pixels. Preserve the
// browser's fractional CSS-pixel offset so the canvas follows high-DPI flings.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = HTMLElement)]
    type ScrollElement;
    #[wasm_bindgen(method, getter, js_name = scrollTop)]
    fn scroll_offset(this: &ScrollElement) -> f64;
    #[wasm_bindgen(method, setter, js_name = scrollTop)]
    fn set_scroll_offset(this: &ScrollElement, value: f64);
}

/// The DOM scroller owns the page offset. These transparent elements only give
/// the browser scroll extent and input regions; Yakui still paints every control.
struct BrowserPage {
    viewport: HtmlElement,
    scroll: HtmlElement,
    content: HtmlElement,
    overlay: HtmlElement,
    bounds: Option<Rect>,
    height: f32,
    drags: Vec<(HtmlElement, Rect)>,
}
impl BrowserPage {
    fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let element = |id| {
            document
                .get_element_by_id(id)
                .unwrap()
                .dyn_into::<HtmlElement>()
        };
        Ok(Self {
            viewport: element("viewport")?,
            scroll: element("page-scroll")?,
            content: element("page-content")?,
            overlay: element("overlay-input")?,
            bounds: None,
            height: 0.,
            drags: Vec::new(),
        })
    }
    fn sync(
        &mut self,
        yak: &yakui::Yakui,
        canvas: &HtmlCanvasElement,
        link: &super::chrome::BrowserScroll,
    ) {
        let origin = Vec2::new(canvas.offset_left() as f32, canvas.offset_top() as f32);
        if let Some(node) = link.widget.get().and_then(|id| yak.layout_dom().get(id)) {
            let bounds = Rect::from_pos_size(node.rect.pos() + origin, node.rect.size());
            if self.bounds != Some(bounds) {
                position(&self.scroll, bounds);
                self.bounds = Some(bounds);
                self.scroll.set_hidden(false);
            }
            let height = link.content_height.get().ceil();
            if self.height != height {
                self.content
                    .style()
                    .set_property("height", &format!("{height}px"))
                    .unwrap();
                self.height = height;
            }
            if let Some(offset) = link.request.take() {
                self.scroll
                    .unchecked_ref::<ScrollElement>()
                    .set_scroll_offset(offset as f64);
            }
        }
        let mut regions = ui::take_pointer_drag_regions(yak);
        // Embedded scroll-area demos and source panels keep their own input.
        // Only the outer page delegates its vertical gestures to the browser.
        fn embedded_scrolls(
            yak: &yakui::Yakui,
            id: yakui::WidgetId,
            page: Rect,
            out: &mut Vec<Rect>,
        ) {
            let Some(layout) = yak.layout_dom().get(id) else {
                return;
            };
            if layout.rect.size().x <= 0. || layout.rect.size().y <= 0. {
                return;
            }
            let node = yak.dom().get(id).unwrap();
            if (&*node.widget as &dyn std::any::Any).is::<yakui::widgets::ScrollableWidget>() {
                let mut rect = layout.rect.constrain(page);
                let mut clip = layout.clipped_by;
                while let Some(id) = clip {
                    let layout = yak.layout_dom().get(id).unwrap();
                    rect = rect.constrain(layout.rect);
                    clip = layout.clipped_by;
                }
                if rect.size().x > 0. && rect.size().y > 0. {
                    out.push(rect);
                }
            }
            for &child in &node.children {
                embedded_scrolls(yak, child, page, out);
            }
        }
        if let Some(node) = link.widget.get().and_then(|id| yak.layout_dom().get(id)) {
            embedded_scrolls(yak, yak.dom().root(), node.rect, &mut regions);
        }
        while self.drags.len() > regions.len() {
            self.drags.pop().unwrap().0.remove();
        }
        for (index, rect) in regions.into_iter().enumerate() {
            let bounds = Rect::from_pos_size(rect.pos() + origin, rect.size());
            if index == self.drags.len() {
                let element = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("div")
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap();
                element.set_class_name("drag-input");
                self.viewport.append_child(&element).unwrap();
                position(&element, bounds);
                self.drags.push((element, bounds));
            } else if self.drags[index].1 != bounds {
                position(&self.drags[index].0, bounds);
                self.drags[index].1 = bounds;
            }
        }
        // Overlay layers own input until they leave. A popup swipe must not pan
        // the page behind it, and outside taps still go to Yakui's backdrop.
        fn overlay_input(yak: &yakui::Yakui, id: yakui::WidgetId) -> (bool, bool) {
            let Some(layout) = yak.layout_dom().get(id) else {
                return (false, false);
            };
            if layout.rect.size().x <= 0. || layout.rect.size().y <= 0. {
                return (false, false);
            }
            let viewport = yak.layout_dom().viewport();
            let mut pointer = layout
                .event_interest
                .intersects(yakui::event::EventInterest::MOUSE_ALL)
                && layout.rect.contains_point(viewport.pos())
                && layout.rect.contains_point(viewport.max());
            let mut overlay = false;
            for &child in &yak.dom().get(id).unwrap().children {
                let (child_overlay, child_pointer) = overlay_input(yak, child);
                overlay |= child_overlay;
                pointer |= child_pointer;
            }
            // Only a full-window pointer backdrop locks the page. Tooltips and
            // hover surfaces also paint in layers, but must not trap page scroll.
            (
                overlay || (id != yak.dom().root() && layout.new_layer && pointer),
                pointer,
            )
        }
        let hidden = !overlay_input(yak, yak.dom().root()).0;
        if self.overlay.hidden() != hidden {
            self.overlay.set_hidden(hidden);
            self.scroll
                .style()
                .set_property("overflow-y", if hidden { "auto" } else { "hidden" })
                .unwrap();
        }
    }
}
fn position(element: &HtmlElement, bounds: Rect) {
    for (name, value) in [
        ("left", bounds.pos().x),
        ("top", bounds.pos().y),
        ("width", bounds.size().x),
        ("height", bounds.size().y),
    ] {
        element
            .style()
            .set_property(name, &format!("{value}px"))
            .unwrap();
    }
}
impl App {
    fn frame(&mut self) {
        self.layout();
        self.renderer
            .paint(&mut self.yak, self.gallery.options.dark);
    }
    fn layout(&mut self) {
        self.gallery.browser_scroll.as_ref().unwrap().offset.set(
            self.browser
                .scroll
                .unchecked_ref::<ScrollElement>()
                .scroll_offset() as f32,
        );
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
        self.browser.sync(
            &self.yak,
            &self.canvas,
            self.gallery.browser_scroll.as_ref().unwrap(),
        );
        if !self.yak.text_input_enabled() && self.keyboard_has_focus() {
            self.focus_surface();
        }
    }
    fn focus_surface(&self) {
        if self.browser.scroll.hidden() {
            focus(&self.canvas);
        } else {
            focus(&self.browser.scroll);
        }
    }
    fn keyboard_has_focus(&self) -> bool {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .active_element()
            .is_some_and(|element| element == **self.keyboard)
    }
    fn editing_key(&mut self, key: KeyCode) {
        for down in [true, false] {
            self.yak.handle_event(Event::KeyChanged {
                key,
                down,
                modifiers: Some(Modifiers::empty()),
            });
        }
    }
    fn cancel_pointer(&mut self) {
        self.touch = None;
        self.yak.handle_event(Event::CursorMoved(None));
        for button in [MouseButton::One, MouseButton::Two, MouseButton::Three] {
            self.yak.handle_event(Event::MouseButtonChanged {
                button,
                down: false,
            });
        }
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
        if down {
            self.gallery
                .browser_scroll
                .as_ref()
                .unwrap()
                .pointer_focus
                .set(false);
        }
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
fn focus(element: &web_sys::HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = element.focus_with_options(&options);
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
    canvas: &web_sys::EventTarget,
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
    let viewport = app.borrow().browser.viewport.clone();
    let state = app.clone();
    listen(&viewport, "pointermove", move |event: PointerEvent| {
        let mut app = state.borrow_mut();
        let position = app.pointer_position(&event);
        if event.pointer_type() == "touch" {
            let scale = app.yak.scale_factor();
            let Some(touch) = app.touch.as_mut().filter(|t| t.id == event.pointer_id()) else {
                return;
            };
            let displacement = position - touch.origin;
            // Vertical page gestures belong entirely to the DOM. The browser
            // sends pointercancel when it takes over and continues after release.
            // Only horizontal preview pans and contained Yakui surfaces use deltas.
            let begin_scroll = !touch.control_drag
                && !touch.scrolling
                && displacement.length() > 8. * scale
                && (!touch.native_page || displacement.x.abs() > displacement.y.abs());
            let mut delta = if begin_scroll {
                touch.origin - position
            } else {
                touch.previous - position
            };
            if touch.native_page {
                delta.y = 0.;
            }
            touch.previous = position;
            touch.scrolling |= begin_scroll;
            let scrolling = touch.scrolling;
            if begin_scroll {
                // Release outside every control before scrolling. A swipe that began
                // on a button must not activate it when the finger is lifted.
                app.yak.handle_event(Event::CursorMoved(None));
                app.yak.handle_event(Event::MouseButtonChanged {
                    button: MouseButton::One,
                    down: false,
                });
            }
            if scrolling {
                // Keep hit testing at the initial contact while the finger moves
                // outside the canvas or over an unrelated control.
                let origin = app.touch.as_ref().unwrap().origin;
                app.yak.handle_event(Event::CursorMoved(Some(origin)));
                app.yak.handle_event(Event::MouseScroll { delta });
                event.prevent_default();
                return;
            }
        }
        app.yak.handle_event(Event::CursorMoved(Some(position)));
    })?;
    let state = app.clone();
    listen(&viewport, "pointerdown", move |event: PointerEvent| {
        if !event.is_primary() {
            return;
        }
        let mut app = state.borrow_mut();
        app.layout();
        let target = event
            .target()
            .unwrap()
            .dyn_into::<web_sys::Element>()
            .unwrap();
        let native_page = target.closest("#page-scroll").ok().flatten().is_some();
        let _ = target.set_pointer_capture(event.pointer_id());
        let position = app.pointer_position(&event);
        // Pointer presses already hit a visible part of the control. Revealing
        // its entire bounds here would move a partly visible slider mid-drag.
        app.gallery
            .browser_scroll
            .as_ref()
            .unwrap()
            .pointer_focus
            .set(true);
        ui::take_pointer_drag_request(&app.yak);
        app.yak.handle_event(Event::CursorMoved(Some(position)));
        if let Some(button) = button(event.button()) {
            app.yak
                .handle_event(Event::MouseButtonChanged { button, down: true });
        }
        let control_drag = ui::take_pointer_drag_request(&app.yak);
        if event.pointer_type() == "touch" {
            app.touch = Some(Touch {
                id: event.pointer_id(),
                origin: position,
                previous: position,
                scrolling: false,
                control_drag,
                native_page,
            });
        } else if !app.keyboard_has_focus() {
            app.focus_surface();
        }
        // Suppress compatibility mouse events stealing focus from the editor
        // after a touch tap. Native panning is governed by CSS touch-action and
        // remains enabled on the page scroller.
        event.prevent_default();
    })?;
    let state = app.clone();
    listen(&viewport, "pointerup", move |event: PointerEvent| {
        if !event.is_primary() {
            return;
        }
        let mut app = state.borrow_mut();
        let touch = app.touch.take();
        if touch.as_ref().is_some_and(|t| t.scrolling) {
            app.yak.handle_event(Event::CursorMoved(None));
        } else {
            let position = app.pointer_position(&event);
            app.yak.handle_event(Event::CursorMoved(Some(position)));
            if let Some(button) = button(event.button()) {
                app.yak.handle_event(Event::MouseButtonChanged {
                    button,
                    down: false,
                });
            }
            // Layout reports text-input ownership. Run it in the trusted input
            // callback so browsers can open their software keyboard immediately.
            app.layout();
            if app.yak.text_input_enabled() {
                focus(&app.keyboard);
            } else {
                app.focus_surface();
            }
            if touch.is_some() {
                app.yak.handle_event(Event::CursorMoved(None));
            }
        }
        if let Some(target) = event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            let _ = target.release_pointer_capture(event.pointer_id());
        }
        event.prevent_default();
    })?;
    let state = app.clone();
    listen(&viewport, "pointerleave", move |_: PointerEvent| {
        let mut app = state.borrow_mut();
        if app.touch.is_none() {
            app.yak.handle_event(Event::CursorMoved(None));
        }
    })?;
    for name in ["pointercancel", "lostpointercapture"] {
        let state = app.clone();
        listen(&viewport, name, move |event: PointerEvent| {
            if event.is_primary() {
                let mut app = state.borrow_mut();
                if name == "pointercancel" || app.touch.is_some() {
                    app.cancel_pointer();
                }
            }
        })?;
    }
    let state = app.clone();
    listen(&viewport, "wheel", move |event: WheelEvent| {
        let mut app = state.borrow_mut();
        let unit = match event.delta_mode() {
            1 => 25.,
            2 => app.canvas.client_height() as f64,
            _ => 1.,
        };
        let scale = web_sys::window().unwrap().device_pixel_ratio() as f32;
        let rect = app.canvas.get_bounding_client_rect();
        app.yak.handle_event(Event::CursorMoved(Some(
            Vec2::new(
                (event.client_x() as f64 - rect.left()) as f32,
                (event.client_y() as f64 - rect.top()) as f32,
            ) * scale,
        )));
        let handled = app.yak.handle_event(Event::MouseScroll {
            delta: Vec2::new(
                (event.delta_x() * unit) as f32,
                (event.delta_y() * unit) as f32,
            ) * scale,
        });
        if handled {
            event.prevent_default();
        }
    })?;
    listen(&viewport, "contextmenu", |event: web_sys::Event| {
        event.prevent_default()
    })?;
    for (name, down) in [("keydown", true), ("keyup", false)] {
        let state = app.clone();
        listen(&viewport, name, move |event: KeyboardEvent| {
            let from_editor = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|element| element.id() == "keyboard");
            if !from_editor {
                state.borrow_mut().key(event, down);
            }
        })?;
    }
    let state = app.clone();
    listen(
        &web_sys::window().unwrap(),
        "blur",
        move |_: web_sys::Event| {
            let mut app = state.borrow_mut();
            for key in std::mem::take(&mut app.keys) {
                app.yak.handle_event(Event::KeyChanged {
                    key,
                    down: false,
                    modifiers: Some(Modifiers::empty()),
                });
            }
            app.cancel_pointer();
            app.yak
                .handle_event(Event::ModifiersChanged(Modifiers::empty()));
        },
    )?;
    let keyboard = app.borrow().keyboard.clone();
    keyboard.set_value(" ");
    let _ = keyboard.set_selection_range(1, 1);
    let state = app.clone();
    listen(&keyboard, "beforeinput", move |event: InputEvent| {
        if event.is_composing() || !event.cancelable() {
            return;
        }
        let key = match event.input_type().as_str() {
            "deleteContentBackward" => Some(KeyCode::Backspace),
            "deleteContentForward" => Some(KeyCode::Delete),
            "insertLineBreak" | "insertParagraph" => Some(KeyCode::Enter),
            _ => None,
        };
        if let Some(key) = key {
            event.prevent_default();
            state.borrow_mut().editing_key(key);
        }
    })?;
    let state = app.clone();
    listen(&keyboard, "input", move |event: InputEvent| {
        if event.is_composing() {
            return;
        }
        let mut app = state.borrow_mut();
        let value = app.keyboard.value();
        // A leading sentinel lets software keyboards emit Backspace even though
        // Cosmic owns the actual document and selection. Disable browser word
        // correction rather than maintaining a second, divergent document here.
        match event.input_type().as_str() {
            "deleteContentBackward" => app.editing_key(KeyCode::Backspace),
            "deleteContentForward" => app.editing_key(KeyCode::Delete),
            "insertLineBreak" | "insertParagraph" => app.editing_key(KeyCode::Enter),
            _ => {
                for c in value.strip_prefix(' ').unwrap_or(&value).chars() {
                    app.yak.handle_event(Event::TextInput(c));
                }
            }
        }
        app.keyboard.set_value(" ");
        let _ = app.keyboard.set_selection_range(1, 1);
    })?;
    for (name, down) in [("keydown", true), ("keyup", false)] {
        let state = app.clone();
        listen(&keyboard, name, move |event: KeyboardEvent| {
            // Text, deletion and Enter arrive through beforeinput/input. Navigation
            // remains a physical-key operation, including Tab and Escape.
            if event.is_composing()
                || matches!(event.key().as_str(), "Backspace" | "Delete" | "Enter")
                || (event.key().chars().count() == 1 && !event.ctrl_key() && !event.meta_key())
            {
                return;
            }
            state.borrow_mut().key(event, down);
        })?;
    }
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
    let mut gallery = Gallery::new(options()?);
    gallery.browser_scroll = Some(Rc::new(Default::default()));
    let app = Rc::new(RefCell::new(App {
        canvas,
        renderer,
        yak: new_yak(),
        gallery,
        browser: BrowserPage::new()?,
        keys: HashSet::new(),
        touch: None,
        keyboard: web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("keyboard")
            .unwrap()
            .dyn_into()?,
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
