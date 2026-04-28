use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::Vec3;
use tracing::{trace, trace_span};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop,
    window::WindowAttributes,
};

use crate::{
    application::render::{Camera, RenderState},
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
    pub main_camera: Camera,
    pub time_of_creation: Instant,
    pub time_of_last_tick: Instant,
    pub frame_time_accumulator: Duration,
}

impl ApplicationState {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let s = trace_span!("application_state");
        let _ = s.enter();

        trace!("Creating new window and render state...");

        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Building Blocks")
                    .with_inner_size(PhysicalSize::new(640, 640)),
            )
            .expect("Failed to create application window!");
        let window = Arc::new(window);

        let universe = Universe::example();
        trace!("Created example universe.");

        let world = World::example();
        trace!("Created example world.");

        let start_transform = Camera::look_at_world_matrix(
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
        );
        let main_camera = Camera {
            transform: start_transform,
            projection: render::CameraProjection::Perspective {
                vertical_fov_radians: 60.0_f32.to_radians(),
                z_near_clipping_plane: 0.1,
                z_far_clipping_plane: 100.0,
            },
        };
        /*let main_camera = Camera {
            transform: start_transform,
            projection: render::CameraProjection::Axonometric {
                scale: 5.0,
                basis: Mat3::from_cols(Vec3::X, Vec3::Y, Vec3::Y + Vec3::Z),
                z_near_clipping_plane: -1000.0,
                z_far_clipping_plane: 1000.0,
            },
        };*/

        let render_state = pollster::block_on(RenderState::new(window, &main_camera));
        trace!("Created render state.");

        trace!("All done, let's run the app now!");

        Self {
            universe,
            world,
            main_camera,
            render_state,

            time_of_creation: Instant::now(),
            time_of_last_tick: Instant::now(),
            frame_time_accumulator: Duration::from_millis(0),
        }
    }

    fn tick(&mut self) {
        let s = trace_span!("World Tick.");
        let _ = s.enter();

        let t = self.time_of_creation.elapsed().as_secs_f32();

        self.main_camera.transform = Camera::look_at_world_matrix(
            Vec3::new(10.0, 10.0, 10.0).rotate_z(t),
            Vec3::new(1.5, 1.5, 1.5),
            Vec3::Z,
        );
    }
}

impl Application {
    pub fn new() -> Self {
        Application { state: None }
    }

    pub fn run() {
        let s = trace_span!("application");
        let _ = s.enter();

        trace!("Initialized.");

        let mut app = Application::new();
        EventLoop::new().unwrap().run_app(&mut app).unwrap();

        trace!("Done.");
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
            universe,
            world,
            main_camera,
            time_of_creation,
            ..
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
                render_state.prepare(universe, world, main_camera, time_of_creation.elapsed());
            }
            WindowEvent::RedrawRequested => render_state.render(&universe, &world),

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
            state.render_state.prepare(
                &mut state.universe,
                &mut state.world,
                &state.main_camera,
                state.time_of_creation.elapsed(),
            );
        }

        // Still requesting to draw as fast as the GPU will allow...
        state.render_state.window.request_redraw();
    }
}
