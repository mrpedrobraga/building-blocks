use std::sync::Arc;

use tracing::{info, info_span};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop,
    window::WindowAttributes,
};

use crate::application::render::RenderState;

pub mod render;

pub struct Application {
    pub render_state: Option<RenderState>,
}

impl Application {
    pub fn new() -> Self {
        Application { render_state: None }
    }

    pub fn run() {
        let s = info_span!("application");
        let _ = s.enter();

        info!("Initialized.");

        let mut app = Application::new();
        EventLoop::new().unwrap().run_app(&mut app).unwrap();

        info!("Done.");
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.render_state.is_none() {
            let s = info_span!("creating_render_state");
            let _ = s.enter();

            info!("Creating new window and render state...");
            let window = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Building Blocks")
                        .with_inner_size(PhysicalSize::new(640, 640)),
                )
                .expect("Failed to create application window!");
            let window = Arc::new(window);
            let render_state = pollster::block_on(RenderState::new(window));
            self.render_state = Some(render_state);
            info!("Success!")
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match self.render_state.as_mut() {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => state.render(&[]),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(state) = &self.render_state {
            state.window.request_redraw();
        }
    }
}
