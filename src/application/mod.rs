use std::sync::Arc;

use tracing::{info, info_span};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop,
    window::WindowAttributes,
};

use crate::{
    application::render::RenderState,
    universe::{Universe, World},
};

pub mod render;

pub struct Application {
    pub state: Option<ApplicationState>,
}

pub struct ApplicationState {
    pub render_state: RenderState,
    pub universe: Universe,
    pub world: World,
}

impl ApplicationState {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let s = info_span!("application_state");
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
        let mut render_state = pollster::block_on(RenderState::new(window));
        info!("Created render state.");

        let mut universe = Universe::example();
        info!("Created example universe.");

        let mut world = World::example();
        info!("Created example world.");

        info!("Preparing GPU for World and Universe...");
        render_state.prepare(&mut universe, &mut world);

        info!("All done, let's run the app now!");

        Self {
            render_state,
            universe,
            world,
        }
    }
}

impl Application {
    pub fn new() -> Self {
        Application { state: None }
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
        if self.state.is_none() {
            self.state = Some(ApplicationState::new(event_loop));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(ApplicationState {
            render_state,
            universe: _,
            world,
        }) = &mut self.state
        else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                render_state.resize(physical_size);
            }
            WindowEvent::RedrawRequested => render_state.render(&world),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.render_state.window.request_redraw();
        }
    }
}
