//! # Render
//!
//! The main struct here is [`RenderClient`] — it keeps track of a universe (with definitions of blocks, materials, etc),
//! and the current world it's rendering. Those are usually pulled from a server currently running a game,
//! but not necessarily — you can render a universe and a world created out of thin air, or snapshots of one loaded from a file.

use std::sync::Arc;

use glam::{UVec2, Vec3};
use tracing::{info, info_span};
use wgpu::{Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions};
use winit::{dpi::PhysicalSize, window::Window};

use crate::client::{
    gui::render::{
        blocks_pipeline::{BlockGroupPipeline, GlobalUniforms},
        camera::Camera,
        render_target::{RenderTarget, TextureViewSet, WindowRenderTarget},
        squares_pipeline::SquaresPipeline,
        views::{BlockGroupRenderView, SceneRenderView, UniverseRenderView},
    },
    GameView,
};

pub mod blocks_pipeline;
pub mod camera;
pub mod render_target;
pub mod squares_pipeline;
pub mod views;

pub struct Gpu {
    device: Device,
    queue: Queue,
}

/// The main struct of this module!
///
/// See the module-level documentation.
pub struct RenderClient {
    pub window: Arc<Window>,
    pub window_render_target: WindowRenderTarget,

    global_uniforms: GlobalUniforms,
    global_uniform_buffer: wgpu::Buffer,

    pub universe_render_view: Option<UniverseRenderView>,
    pub current_scene_render_view: Option<SceneRenderView>,

    pub squares_pipeline: Option<SquaresPipeline>,
    pub block_group_pipeline: Option<BlockGroupPipeline>,

    // TODO: Actually, maybe the render client shouldn't be the one holding the GPU connection...
    // `Application` might do it instead.
    gpu: Gpu,
}

impl RenderClient {
    /// Creates a new render client holding onto a window.
    pub async fn new(window: Arc<Window>) -> Self {
        // First we connect to the GPU.
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        // We create a surface from the window which we'll render to.
        let surface = instance.create_surface(window.clone()).unwrap();
        // We request an adapter compatible with the surface.
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];

        // And here we get out GPU connections.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let gpu = Gpu { device, queue };

        // We create an empty uniform buffer (we'll write to it before rendering for the first time, dw!)
        let global_uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let window_render_target =
            WindowRenderTarget::new(window.inner_size(), surface, surface_format, &gpu.device);

        Self {
            window,
            window_render_target,
            gpu,
            global_uniforms: GlobalUniforms::default(),
            global_uniform_buffer,
            universe_render_view: None,
            current_scene_render_view: None,
            squares_pipeline: None,
            block_group_pipeline: None,
        }
    }

    /// Changes the size of the render target
    /// TODO: In the future, `RenderClient` won't hold onto a single render target.
    pub fn resize(&mut self, new_size: UVec2) {
        self.window_render_target
            .resize(&self.gpu.device, PhysicalSize::new(new_size.x, new_size.y));

        // TODO: Resize the layers in the GUI pipeline, yeah!
        // will look like `self.squares_pipeline.resize(&self.gpu.device, new_size)`;

        // TODO: Obviously we'll access this camera from somewhere instead of creating it here.
        let c = Camera {
            transform: Camera::make_look_at_matrix(
                Vec3::new(10.0, 10.0, 10.0).rotate_z(0.0),
                Vec3::new(1.5, 1.5, 1.5),
                Vec3::Z,
            ),
            projection: camera::CameraProjection::Perspective {
                vertical_fov_radians: 60.0_f32.to_radians(),
                z_near_clipping_plane: 0.1,
                z_far_clipping_plane: 100.0,
            },
        };
        self.global_uniforms = GlobalUniforms {
            view_matrix: c.view_matrix(new_size).to_cols_array(),
            ..self.global_uniforms
        };
        self.sync_uniforms();
    }

    /// Syncs the uniforms to the GPU!
    pub fn sync_uniforms(&self) {
        self.gpu.queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.global_uniforms]),
        );
    }

    /// Creates render views from a client's view of a world.
    pub fn prepare_from_scratch(&mut self, game_view: &GameView) {
        let s = info_span!("render client preparing from scratch");
        let _ = s.enter();

        let screen_size = UVec2::new(
            self.window_render_target.surface_size.width,
            self.window_render_target.surface_size.height,
        );

        /* Creating 2D information!!! */

        self.squares_pipeline = Some(SquaresPipeline::new(
            &self.gpu,
            self.window_render_target.surface_config.format,
            screen_size,
        ));

        /* Creating 3D information!!! */

        info!("Creating bind group layout.");
        // It's possible to create these only once for ever I'm sure.

        let universe_bind_group_layout = UniverseRenderView::bind_group_layout(&self.gpu);
        let block_group_bind_group_layout = BlockGroupRenderView::bind_group_layout(&self.gpu);

        // TODO: Get the camera from somewhere else lol.
        let c = Camera {
            transform: Camera::make_look_at_matrix(
                Vec3::new(10.0, 10.0, 10.0).rotate_z(0.0),
                Vec3::new(1.5, 1.5, 1.5),
                Vec3::Z,
            ),
            projection: camera::CameraProjection::Perspective {
                vertical_fov_radians: 60.0_f32.to_radians(),
                z_near_clipping_plane: 0.1,
                z_far_clipping_plane: 100.0,
            },
        };
        self.global_uniforms = GlobalUniforms {
            view_matrix: c.view_matrix(screen_size).to_cols_array(),
            global_time: 0.0,
            _padding: [0.0; 3],
        };
        self.sync_uniforms();

        info!("Creating render views!");
        // These will be createed once but should be able to be updated incrementally
        // especially through streaming...
        //
        // Ideally you'd get the correct "capacity" from upstream
        // and then progressively send data to the gpu in another thread.

        let universe_render_view = UniverseRenderView::new(
            &self.gpu,
            &self.global_uniform_buffer,
            &game_view.current_universe,
            &universe_bind_group_layout,
        );
        let current_scene_render_view = SceneRenderView::new(
            &self.gpu,
            &game_view.current_scene,
            &block_group_bind_group_layout,
        );

        self.universe_render_view = Some(universe_render_view);
        self.current_scene_render_view = Some(current_scene_render_view);

        self.block_group_pipeline = Some(BlockGroupPipeline::new(
            &self.gpu.device,
            self.window_render_target.surface_config.format,
            universe_bind_group_layout,
            block_group_bind_group_layout,
        ));
    }

    /// Draws onto the render target!
    pub fn draw(&mut self) {
        let s = info_span!("render client draw");
        let _ = s.enter();

        // TODO: In the future, a texture view set will be passed as an argument.
        let texture_view_set = self
            .window_render_target
            .texture_view_set()
            .expect("Couldn't get target texture view");

        // The command encoder, our beloved, is how we schedule commands to be sent to the GPU.
        let mut command_encoder = self.gpu.device.create_command_encoder(&Default::default());

        let mut _render_pass = self.create_render_pass(&mut command_encoder, &texture_view_set);

        // TODO: Go through the different pipelines rendering everything.

        // - Render skybox (data-driven shader)
        // - Render tiles (data-driven shader)
        // - Render sprites (data-driven shader)
        // - Render particles (data-driven shader)
        // - Custom rendering (data-driven pipeline)
        // - World post-processing (data-driven shader)
        // - Render Gizmos and GUI

        if let Some(universe_render_view) = &self.universe_render_view {
            _render_pass.set_bind_group(0, Some(&universe_render_view.bind_group), &[]);

            if let Some(block_group_pipeline) = &self.block_group_pipeline {
                _render_pass.set_pipeline(&block_group_pipeline.render_pipeline);

                if let Some(scene_render_view) = &self.current_scene_render_view {
                    for block_group in &scene_render_view.block_groups {
                        _render_pass.set_bind_group(1, Some(&block_group.bind_group), &[]);
                        _render_pass.draw(0..36, 0..block_group.volume());
                    }
                }
            }
        }

        if let Some(squares_pipeline) = &self.squares_pipeline {
            _render_pass.set_bind_group(
                0,
                Some(&squares_pipeline.render_to_screen_bind_group),
                &[],
            );
            _render_pass.set_bind_group(1, None, &[]);
            _render_pass.set_pipeline(&squares_pipeline.render_to_screen_pipeline);
            _render_pass.draw(0..6, 0..1);
        }

        drop(_render_pass);

        // Send all commands to the GPU.
        self.gpu
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        // TODO: In the future, presenting should be handled at by the caller of this function.
        if let Some(surface_texture) = texture_view_set.surface {
            surface_texture.present();
        };
    }

    fn create_render_pass<'pass>(
        &self,
        command_encoder: &'pass mut wgpu::CommandEncoder,
        texture_view_set: &TextureViewSet,
    ) -> wgpu::RenderPass<'pass> {
        // TODO: Instead of a clear colour, in the future, I'll render a sky box.
        // Actually, the sky will be white and the floor will be a fractal checkerboard pattern.
        //
        // A light clear colour, or even a dark one but not pure black, is more "fun" and "toyish" and less intimidating.
        let clear_color = wgpu::Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };

        let albedo = wgpu::RenderPassColorAttachment {
            view: &texture_view_set.albedo,
            ops: wgpu::Operations {
                // LoadOp "don't care" could be used here, since we'll always draw the void.
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
            // Not used since the albedo texture isn't 3D.
            depth_slice: None,
            // No multisampling is used, so this is set to None.
            resolve_target: None,
        };

        let depth = wgpu::RenderPassDepthStencilAttachment {
            view: &texture_view_set.depth,
            depth_ops: Some(wgpu::Operations {
                // This is 1.0 (infinitely far) by default.
                // The "skybox" floor, even though it's rendered,
                // will never actually occlude anything in the scene, you see.
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            // The stencil buffer is unused.
            stencil_ops: None,
        };

        command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(albedo)],
            depth_stencil_attachment: Some(depth),

            // TODO: Enable this for accurate GPU profiling!
            timestamp_writes: None,
            // TODO: Enable this for multiview rendering situations such as VR!
            // In VR, the render target will be an ArrayTexture with two layers...
            // The shader will differentiate between the left and right eye and slightly translate the world
            // based on the camera's setting for eye separation.
            multiview_mask: None,
            // This is for querying "actual" visibility, that is, if the GPU actually rendered certain objects.
            // I don't think I'll ever use this.
            occlusion_query_set: None,
        })
    }
}
