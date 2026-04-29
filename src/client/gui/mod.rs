use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::UVec2;
use tracing::{info, info_span};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop,
    window::WindowAttributes,
};

use crate::{
    client::{gui::render::RenderClient, GameView, GuiClient},
    server::ServerAdapter,
    universe::{Universe, World},
};

pub mod render;

pub struct Application<'app> {
    // TODO: Maybe use a less exclusive reference, like an Arc+Mutex?
    // Right now it's expected that the application has full control over the client, you see...
    pub client: &'app mut GuiClient,
    pub state: Option<ApplicationState>,
}

pub struct ApplicationState {
    pub render_client: RenderClient,
    // TODO: Maybe move these to the RenderClient itself,
    // and in the application just call `RenderClient::tick`
    // and then the client handles FPS and whatnot.
    pub time_of_creation: Instant,
    pub time_of_last_tick: Instant,
    pub frame_time_accumulator: Duration,
}

impl ApplicationState {
    fn new(client: &mut GuiClient, event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let s = info_span!("application_state");
        let _ = s.enter();

        info!("Creating new window and render client...");

        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Building Blocks")
                    .with_inner_size(PhysicalSize::new(640, 640)),
            )
            .expect("Failed to create application window!");
        let window = Arc::new(window);
        let mut render_client = pollster::block_on(RenderClient::new(window));

        info!("Gathering information from the server...");

        ApplicationState::prepare_from_scratch_for_server(&mut render_client, client);

        info!("All done, let's run the app now!");

        Self {
            render_client,
            time_of_creation: Instant::now(),
            time_of_last_tick: Instant::now(),
            frame_time_accumulator: Duration::from_millis(0),
        }
    }

    fn prepare_from_scratch_for_server(render_client: &mut RenderClient, client: &mut GuiClient) {
        if let Some(_server_adapter) = &client.server_adapter {
            // // TODO: Get resources from the server on another task,
            // // using some very granular multi-threading!
            let universe = _server_adapter.get_universe().unwrap();

            // World creation and world loading will be both dictated by `Universe`.
            // It can be as simple as duplicating a template, and as complex as generating terrain.
            let world = World::example();
            // Loading a first scene from a world will also be decided by the Universe.
            let scene = world.scenes.get("default").cloned().unwrap();

            let game_view = GameView {
                current_universe: universe,
                current_world: world,
                current_scene: scene,
            };

            render_client.prepare_from_scratch(&game_view);

            client.game_resources = Some(game_view);
        } else {
            info!("No server connected.");
        }
    }

    fn tick(&mut self) {
        // Nothing here yet!
    }
}

impl<'app> Application<'app> {
    pub fn new(client: &'app mut GuiClient) -> Self {
        Application {
            client,
            state: None,
        }
    }

    pub fn run(client: &'app mut GuiClient) {
        let s = info_span!("application");
        let _ = s.enter();

        info!("Initialized.");

        let mut app = Application::new(client);
        EventLoop::new().unwrap().run_app(&mut app).unwrap();

        info!("Done.");
    }
}

impl<'app> ApplicationHandler for Application<'app> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_none() {
            self.state = Some(ApplicationState::new(&mut self.client, event_loop));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(ApplicationState { render_client, .. }) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                render_client.resize(UVec2::new(physical_size.width, physical_size.height));
            }
            WindowEvent::RedrawRequested => render_client.draw(),

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(state) = &mut self.state else { return };

        let now = Instant::now();
        let time_since_last_tick = now.duration_since(state.time_of_last_tick);
        state.time_of_last_tick = now;

        // We accumulate time into a time accumulator...
        // Though we want to retroactively compute frames that we lost,
        // if the `time_since_last_tick` was too large we will just slow down.
        state.frame_time_accumulator += time_since_last_tick.min(Duration::from_millis(100));

        let target_fps = 60.0;
        let expected_tick_duration = Duration::from_secs_f64(1.0 / target_fps);

        // For each multiple of `expected_tick_duration` that has passed
        // we execute one (1) tick!
        // At the end, a tiny leftover might be stored that gives us a head
        // on the next `about_to_wait` call...
        if state.frame_time_accumulator >= expected_tick_duration {
            while state.frame_time_accumulator >= expected_tick_duration {
                state.tick();
                state.frame_time_accumulator -= expected_tick_duration;
            }
            // TODO: Update the render client of how much in-world time has passed.
            // The render client shouldn't rely on real-world time, you see,
            // since it may be rendering in a locked frame-rate.
        }

        // Still requesting to draw as fast as the GPU will allow...
        state.render_client.window.request_redraw();
    }
}
