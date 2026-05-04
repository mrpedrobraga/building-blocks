use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::UVec2;
use smol::channel::Sender;
use tracing::{info, info_span};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    client::{
        app::render::{render_target::WindowRenderTarget, Gpu, RenderClient},
        gui::{AppMessage, Client},
        GameView,
    },
    data_packs::Universe,
    world::World,
};

pub mod render;

pub struct Application {
    // TODO: Maybe use a less exclusive reference, like an Arc+Mutex?
    // Right now it's expected that the application has full control over the client, you see...
    pub gui_message_tx: Sender<AppMessage>,
    pub state: Option<ApplicationState>,
}

pub struct ApplicationState {
    gpu: Gpu,
    pub window: Arc<Window>,
    pub render_client: RenderClient,
    // TODO: Maybe move these to the RenderClient itself,
    // and in the application just call `RenderClient::tick`
    // and then the client handles FPS and whatnot.
    pub time_of_creation: Instant,
    pub time_of_last_tick: Instant,
    pub frame_time_accumulator: Duration,
}

impl ApplicationState {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
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

        // First we connect to the GPU.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        // We create a surface from the window which we'll render to.
        let surface = instance.create_surface(window.clone()).unwrap();
        // We request an adapter compatible with the surface.
        let adapter = smol::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];

        // And here we get out GPU connections.
        let (device, queue) =
            smol::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
        let gpu = smol::block_on(Gpu::new(device, queue));

        let render_target =
            WindowRenderTarget::new(window.inner_size(), surface, surface_format, &gpu.device);

        let render_client = pollster::block_on(RenderClient::new(&gpu, render_target));

        info!("Gathering information from the server...");

        info!("All done, let's run the app now!");

        Self {
            window,
            render_client,
            time_of_creation: Instant::now(),
            time_of_last_tick: Instant::now(),
            frame_time_accumulator: Duration::from_millis(0),
            gpu,
        }
    }

    #[deprecated]
    #[allow(unused)]
    fn prepare_from_scratch_for_server(
        &self,
        render_client: &mut RenderClient,
        client: &mut Client,
    ) {
        if let Some(server) = &client.server_interface {
            // TODO: Stream resources from the server on another thread
            // so we don't lag while waiting for resources.
            let universe = Universe::example();

            // So, we're creating a dummy world here...
            //
            // In reality, joining a world will be a message incoming from the server.
            let world = World::example();
            // Loading a first scene from a world will also be decided by the Universe.
            let scene = world.scenes.get("default").cloned().unwrap();

            let game_view = GameView {
                current_universe: universe,
                current_world: world,
                current_scene: scene,
            };

            // TODO: Introduce granular updates to the content of the render client.
            #[allow(deprecated)]
            render_client.prepare_from_scratch(&self.gpu, &game_view);

            client.game_resources = Some(game_view);
        } else {
            info!("No server connected.");
        }
    }

    fn tick(&mut self) {
        // Nothing here yet!
    }
}

impl Application {
    pub fn new(gui_message_tx: Sender<AppMessage>) -> Self {
        Application {
            gui_message_tx,
            state: None,
        }
    }

    pub fn run(&mut self) {
        let s = info_span!("application");
        let _ = s.enter();

        info!("Initialized.");

        EventLoop::new().unwrap().run_app(self).unwrap();

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
            render_client, gpu, ..
        }) = &mut self.state
        else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                render_client.resize(gpu, UVec2::new(physical_size.width, physical_size.height));
            }
            WindowEvent::RedrawRequested => {
                // TODO: Maybe send something like a "redraw" message?
                render_client.draw(gpu)
            }
            event => {
                // TODO: Think about this better.
                let _ = self.gui_message_tx.send(AppMessage::WindowEvent(event));
                //.expect("Failed to send window event to client?");
            }
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
        state.window.request_redraw();
    }
}
