//! # Render
//!
//! The main struct here is [`RenderClient`] — it keeps track of a universe (with definitions of blocks, materials, etc),
//! and the current world it's rendering. Those are usually pulled from a server currently running a game,
//! but not necessarily — you can render a universe and a world created out of thin air, or snapshots of one loaded from a file.

use std::sync::Arc;

use glam::{UVec2, UVec3};
use image::{ImageBuffer, Rgba};
use wgpu::{Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    block::{Block, BlockDefinition},
    client::{
        gui::render::{
            pipeline::{BlockGroupPipeline, GlobalUniforms},
            render_target::{RenderTarget, TextureViewSet, WindowRenderTarget},
            views::{SceneRenderView, UniverseRenderView},
        },
        GameView,
    },
};

pub mod camera;
pub mod pipeline;
pub mod render_target;
pub mod tmp;
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

    #[allow(unused)]
    pipeline: BlockGroupPipeline,
    global_uniforms: GlobalUniforms,
    global_uniform_buffer: wgpu::Buffer,

    universe_render_view: Option<UniverseRenderView>,
    current_scene_render_view: Option<SceneRenderView>,

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

        let pipeline = BlockGroupPipeline::new(&gpu.device, surface_format);

        Self {
            pipeline,
            window,
            window_render_target,
            gpu,
            global_uniforms: GlobalUniforms::default(),
            global_uniform_buffer,
            universe_render_view: None,
            current_scene_render_view: None,
        }
    }

    /// Changes the size of the render target
    /// TODO: In the future, `RenderClient` won't hold onto a single render target.
    pub fn resize(&mut self, new_size: UVec2) {
        self.window_render_target
            .resize(&self.gpu.device, PhysicalSize::new(new_size.x, new_size.y));
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
        self.universe_render_view = Some(UniverseRenderView::new(
            &self.gpu,
            &game_view.current_universe,
        ));
    }

    /// Draws onto the render target!
    pub fn draw(&mut self) {
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

// pub struct RenderClientGpu {
//     device: Device,
//     queue: Queue,
//     target: WindowRenderTarget,
//     pipeline: BlocksPipeline,
//     pub window: Arc<Window>,
//     uniforms: GlobalUniforms,
//     global_uniform_buffer: wgpu::Buffer,
// }

// impl RenderClientGpu {
//     pub async fn new(window: Arc<Window>, camera: &Camera) -> Self {
//         let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
//         let surface = instance.create_surface(window.clone()).unwrap();
//         let adapter = instance
//             .request_adapter(&RequestAdapterOptions {
//                 compatible_surface: Some(&surface),
//                 ..Default::default()
//             })
//             .await
//             .unwrap();
//         let (device, queue) = adapter
//             .request_device(&wgpu::DeviceDescriptor::default())
//             .await
//             .unwrap();

//         let surface_capabilities = surface.get_capabilities(&adapter);
//         let surface_format = surface_capabilities.formats[0];

//         let target = WindowRenderTarget::new(window.inner_size(), surface, surface_format, &device);
//         let pipeline = BlocksPipeline::new(&device, surface_format);

//         let uniforms = GlobalUniforms {
//             view_matrix: camera
//                 .view_matrix(UVec2::new(
//                     target.surface_config.width,
//                     target.surface_config.height,
//                 ))
//                 .to_cols_array(),
//             global_time: 0.0,
//             x2: 0.0,
//             x3: 0.0,
//             x4: 0.0,
//         };

//         let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Cluster Uniform Buffer"),
//             contents: bytemuck::cast_slice(&[uniforms]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         });

//         RenderClientGpu {
//             device,
//             queue,
//             pipeline,
//             target,
//             window,
//             uniforms,
//             global_uniform_buffer,
//         }
//     }

//     pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
//         self.target.configure(&self.device, new_size);
//     }

//     pub fn update(&mut self, camera: &Camera, global_time: Duration) {
//         self.update_uniforms(camera, global_time);
//     }

//     pub fn update_uniforms(&mut self, camera: &Camera, global_time: Duration) {
//         self.uniforms = GlobalUniforms {
//             view_matrix: camera
//                 .view_matrix(UVec2::new(
//                     self.target.surface_config.width,
//                     self.target.surface_config.height,
//                 ))
//                 .to_cols_array(),
//             global_time: global_time.as_secs_f32(),
//             x2: 0.0,
//             x3: 0.0,
//             x4: 0.0,
//         };

//         self.queue.write_buffer(
//             &self.global_uniform_buffer,
//             0,
//             bytemuck::cast_slice(&[self.uniforms]),
//         );
//     }

//     pub fn prepare(
//         &mut self,
//         universe: &mut Universe,
//         world: &mut World,
//         camera: &Camera,
//         global_time: Duration,
//     ) {
//         self.update(camera, global_time);

//         if universe.gpu.is_none() {
//             universe.gpu = Some(Self::create_universe_gpu_resources(
//                 &self,
//                 universe,
//                 &self.pipeline,
//             ))
//         }

//         for (_, block_cluster) in world.block_clusters.iter_mut() {
//             if block_cluster.gpu.is_none() {
//                 block_cluster.gpu = Some(Self::create_block_cluster_gpu_resources(
//                     &self,
//                     block_cluster,
//                     universe
//                         .gpu
//                         .as_ref()
//                         .map(|x| &x.block_definitions_buffer)
//                         .unwrap(),
//                     &self.pipeline,
//                     // TODO: Move this out of here into a different set of uniforms
//                     // that is global and not per-cluster!
//                 ));
//             }
//         }
//     }

//     pub fn render(&mut self, universe: &Universe, world: &World) {
//         let s = info_span!("render");
//         let _ = s.enter();

//         let view = self
//             .target
//             .texture_view()
//             .expect("Failed to get current target texture to render to...");

//         let mut encoder = self
//             .device
//             .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                 label: Some("Render Encoder"),
//             });

//         {
//             let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Main Render Pass"),
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &view,
//                     depth_slice: None,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color {
//                             r: 0.9,
//                             g: 0.9,
//                             b: 0.9,
//                             a: 1.0,
//                         }),
//                         store: wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
//                     view: &self.target.depth_texture_view,
//                     depth_ops: Some(wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(1.0),
//                         store: wgpu::StoreOp::Store,
//                     }),
//                     stencil_ops: None,
//                 }),
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//                 multiview_mask: None,
//             });

//             render_pass.set_pipeline(&self.pipeline.render_pipeline);

//             render_world(universe, world, render_pass);
//         }

//         self.queue.submit(std::iter::once(encoder.finish()));

//         match self.target.surface.get_current_texture() {
//             wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture.present(),
//             wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture.present(),
//             _ => {
//                 warn!("Surface couldn't be presented.")
//             }
//         };
//     }
// }

// fn render_world(universe: &Universe, world: &World, mut render_pass: wgpu::RenderPass<'_>) {
//     if let Some(universe_gpu) = &universe.gpu {
//         render_pass.set_bind_group(1, &universe_gpu.bind_group, &[]);

//         for (_, block_cluster) in world.block_clusters.iter() {
//             if let Some((_, block_cluster_gpu)) = &block_cluster.gpu {
//                 render_block_cluster(&mut render_pass, block_cluster_gpu);
//             }
//         }
//     }
// }

// fn render_block_cluster(
//     render_pass: &mut wgpu::RenderPass<'_>,
//     block_cluster_gpu: &crate::block::BlockClusterRenderResources,
// ) {
//     render_pass.set_bind_group(0, &block_cluster_gpu.bind_group, &[]);
//     render_pass.draw(0..36, 0..block_cluster_gpu.num_voxels);
// }
