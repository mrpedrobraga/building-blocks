//! # Render
//!
//! Render engine that renders everything to the screen :-)

use std::sync::Arc;

use glam::{Mat4, Quat, UVec2, Vec3};
use tracing::{info_span, warn};
use wgpu::{util::DeviceExt, Device, Instance, InstanceDescriptor, Queue, RequestAdapterOptions};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    client::application::render::{
        pipeline::{BlocksPipeline, GlobalUniforms},
        render_target::{RenderTarget, WindowRenderTarget},
    },
    universe::{Universe, World},
};

pub mod camera;
pub mod pipeline;
pub mod render_target;
pub mod tmp;

pub struct RenderClient {
    pub window: Arc<Window>,
    pub window_render_target: WindowRenderTarget,
    // TODO: Actually, maybe the render client shouldn't be the one holding the GPU connection...
    // Application might do it instead.
    gpu: Gpu,
}

pub struct Gpu {
    pipeline: BlocksPipeline,
    device: Device,
    queue: Queue,
    global_uniforms: GlobalUniforms,
    global_uniform_buffer: wgpu::Buffer,
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

        // We create an empty uniform buffer (we'll write to it before rendering for the first time, dw!)
        let global_uniforms = GlobalUniforms {
            view_matrix: Mat4::from_rotation_translation(Quat::default(), Vec3::new(0.0, 0.0, 0.0))
                .to_cols_array(),
            global_time: 0.0,
            x2: 0.0,
            x3: 0.0,
            x4: 0.0,
        };

        let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cluster Uniform Buffer"),
            contents: bytemuck::cast_slice(&[global_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline = BlocksPipeline::new(&device, surface_format);

        let gpu = Gpu {
            pipeline,
            device,
            queue,
            global_uniforms,
            global_uniform_buffer,
        };

        let window_render_target =
            WindowRenderTarget::new(window.inner_size(), surface, surface_format, &gpu.device);

        Self {
            window,
            window_render_target,
            gpu,
        }
    }

    /// Changes the size of the render target
    /// TODO: In the future, `RenderClient` won't hold onto a single render target.
    pub fn resize(&mut self, new_size: UVec2) {
        self.window_render_target
            .resize(&self.gpu.device, PhysicalSize::new(new_size.x, new_size.y));
    }

    /// Draws onto the render target!
    pub fn draw(&mut self) {
        // Nothing here yet!
    }
}

impl Gpu {
    /// Syncs the uniforms to the GPU!
    pub fn sync_uniforms(&self) {
        self.queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.global_uniforms]),
        );
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
