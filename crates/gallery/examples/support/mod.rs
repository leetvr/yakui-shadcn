//! Window boilerplate shared by these standalone consuming examples.
use std::error::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use yakui_adequate_ui as ui;
struct App<F> {
    build: F,
    yak: yakui::Yakui,
    window: Option<Window>,
    graphics: Option<yakui_app::Graphics>,
}
impl<F: FnMut()> ApplicationHandler for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Adequate UI example")
                    .with_inner_size(winit::dpi::LogicalSize::new(900., 700.)),
            )
            .unwrap();
        window.set_ime_allowed(true);
        self.graphics = Some(pollster::block_on(yakui_app::Graphics::new(&window, 4)));
        window.request_redraw();
        self.window = Some(window);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let graphics = self.graphics.as_mut().unwrap();
        graphics.handle_window_event(&mut self.yak, &event, event_loop);
        match event {
            WindowEvent::RedrawRequested => {
                let viewport = self.yak.surface_size() / self.yak.scale_factor();
                self.yak.start();
                ui::with_theme(ui::Theme::light(), || {
                    ui::size(viewport, || {
                        ui::surface(ui::theme().background, None, 0., 24., || {
                            yakui::align(yakui::Alignment::TOP_LEFT, || (self.build)());
                        })
                    });
                });
                self.yak.finish();
                graphics.is_init = false;
                graphics.paint(&mut self.yak, wgpu::Color::WHITE);
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => graphics.resize(size),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}
pub fn run(build: impl FnMut() + 'static) -> Result<(), Box<dyn Error>> {
    let mut yak = yakui::Yakui::new();
    ui::add_fonts(&mut yak);
    EventLoop::new()?.run_app(&mut App {
        build,
        yak,
        window: None,
        graphics: None,
    })?;
    Ok(())
}
