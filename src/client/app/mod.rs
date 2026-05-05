//! # Application
//!
//! For GUI clients, a window will be created with `winit` that will forward inputs to the client's thread,
//! and transparently present whatever the client renders.
//!
//! Note that [`Application`] hijacks (and therefore must be run on) the main thread and, thus,
//! isn't suitable for heavy multithreading work. So on the main thread, nothing is run but [`Application`],
//! and it does nothing but interface with the user.

use crate::client::render::{gpu::Gpu, render_target::window::WindowRenderTarget};
use glam::UVec2;
use smol::channel::Sender;
use std::sync::Arc;
use tracing::{error, info, info_span};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

pub struct Application {
    pub gui_message_tx: Sender<AppMessage>,
    pub window: Option<Arc<Window>>,
}

#[derive(Debug)]
pub enum AppMessage {
    WindowEvent(winit::event::WindowEvent),
    DeviceEvent(winit::event::DeviceEvent),
    SetRenderTarget(Gpu, WindowRenderTarget),
    ResizeRenderTarget(UVec2),
    PleaseRender,
}

impl Application {
    pub fn run(gui_message_tx: Sender<AppMessage>) {
        let s = info_span!("application");
        let _ = s.enter();

        info!("Initialized.");

        EventLoop::new()
            .unwrap()
            .run_app(&mut Application {
                gui_message_tx,
                window: None,
            })
            .unwrap();

        info!("Done.");
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            /* Create the window that catches inputs and renders whatever the client renders. */
            let starting_size = PhysicalSize::new(640, 640);
            let window = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Building Blocks")
                        .with_inner_size(starting_size),
                )
                .expect("Failed to create application window!");
            let window = Arc::new(window);
            self.window = Some(window.clone());

            /* From the window, create a render target that the client can reder to with the GPU. */
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let surface = instance.create_surface(window).unwrap();
            let adapter = smol::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .unwrap();
            let surface_capabilities = surface.get_capabilities(&adapter);
            let surface_format = surface_capabilities.formats[0];
            let (device, queue) =
                smol::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
            let gpu = smol::block_on(Gpu::new(device, queue));
            let _render_target =
                WindowRenderTarget::new(starting_size, surface, surface_format, &gpu.device);

            /* Give the client the render target or close. */
            let message = AppMessage::SetRenderTarget(gpu, _render_target);
            if smol::block_on(self.gui_message_tx.send(message)).is_err() {
                error!(
                    "[App] Failed to initialize — couldn't set the render target on the client..."
                );
                event_loop.exit();
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // TODO: Maybe notify the client of the close request and then if
                // the client agrees we quit.
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                let message =
                    AppMessage::ResizeRenderTarget(UVec2::new(new_size.width, new_size.height));
                if smol::block_on(self.gui_message_tx.send(message)).is_err() {
                    error!("[App] Failed to request the client to resize...");
                    // TODO: Maybe quit?
                    // Concurrency is complex so I don't yet know what it means if this channel
                    // fails to send a message.
                };
            }
            WindowEvent::RedrawRequested => {
                let message = AppMessage::PleaseRender;
                if smol::block_on(self.gui_message_tx.send(message)).is_err() {
                    error!("[App] Failed to request the client to redraw...");
                    // TODO: Maybe quit?
                    // Concurrency is complex so I don't yet know what it means if this channel
                    // fails to send a message.
                };
            }
            event => {
                let message = AppMessage::WindowEvent(event);
                if smol::block_on(self.gui_message_tx.send(message)).is_err() {
                    error!("[App] Failed to request the client to redraw...");
                    // TODO: Maybe quit?
                    // Concurrency is complex so I don't yet know what it means if this channel
                    // fails to send a message.
                };
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
