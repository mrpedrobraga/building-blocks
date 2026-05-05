use crate::client::render::render_target::RenderTarget;
use glam::UVec2;
use tracing::info;
use wgpu::{Device, Queue};

use super::{
    BlockGroupRenderState, GameRenderState, UniverseRenderState,
    pipeline::voxels::VoxelPipeline,
    render_target::{TextureViewSet, window::WindowRenderTarget},
};

#[derive(Debug)]
pub struct Gpu {
    pub device: Device,
    pub queue: Queue,
}

impl Gpu {
    pub async fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self { device, queue }
    }
}

pub struct GameRenderResources {
    pub gpu: Gpu,
    pub render_target: WindowRenderTarget,
    pub voxel_pipeline: VoxelPipeline,
}

impl GameRenderResources {
    pub fn new(gpu: Gpu, render_target: WindowRenderTarget) -> Self {
        let universe_bind_group_layout = UniverseRenderState::bind_group_layout(&gpu);
        let block_group_bind_group_layout = BlockGroupRenderState::bind_group_layout(&gpu);

        let voxel_pipeline = VoxelPipeline::new(
            &gpu.device,
            render_target.surface_config.format,
            universe_bind_group_layout,
            block_group_bind_group_layout,
        );

        GameRenderResources {
            gpu,
            render_target,
            voxel_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: UVec2) {
        info!("[Client] Resized to {:?}.", new_size);

        self.render_target.resize(
            &self.gpu.device,
            winit::dpi::PhysicalSize::new(new_size.x, new_size.y),
        );

        // TODO: Update and sync uniforms that depend on screen size.
    }

    pub fn render(&mut self, _state: &GameRenderState) {
        let target_texture_set = self
            .render_target
            .texture_view_set()
            .expect("Couldn't get current texture to render to...");

        let mut command_encoder =
            self.gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Main Command Encoder"),
                });

        let render_pass = self.render_pass(&mut command_encoder, &target_texture_set);

        GameRenderResources::draw_sequence(render_pass);

        self.gpu
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        if let Some(surface_texture) = target_texture_set.surface {
            surface_texture.present();
        }
    }

    fn draw_sequence<'pass>(_render_pass: wgpu::RenderPass<'pass>) {
        // TODO: Here we do all our draw calls...
    }

    fn render_pass<'pass>(
        &self,
        command_encoder: &'pass mut wgpu::CommandEncoder,
        target_texture_set: &TextureViewSet,
    ) -> wgpu::RenderPass<'pass> {
        /* TODO: Maybe make this configurable? Who knows... */
        let clear_color = wgpu::Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };

        let main_color_attachment = wgpu::RenderPassColorAttachment {
            view: &target_texture_set.albedo,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
            resolve_target: None,
        };

        let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view: &target_texture_set.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0), // infinitely far away
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None, // TODO: Maybe use this for whatever?
        };

        command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(main_color_attachment)],
            depth_stencil_attachment: Some(depth_stencil_attachment), // TODO: Use this!
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None, // TODO: Use this for VR?
        })
    }
}

impl UniverseRenderState {
    pub fn bind_group_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
        /*
            @group(0) @binding(0) var<uniform> globals: GlobalUniforms;
            @group(0) @binding(1) var<storage, read> block_definitions: array<BlockDefinition>;
            @group(0) @binding(2) var material_atlas: texture_2d<f32>;
            @group(0) @binding(3) var material_atlas_s: sampler;
        */
        let global_uniforms_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let block_definitions_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture_atlas_view_entry = wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let texture_atlas_sampler_entry = wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Universe Bind Group Layout"),
                entries: &[
                    global_uniforms_entry,
                    block_definitions_entry,
                    texture_atlas_view_entry,
                    texture_atlas_sampler_entry,
                ],
            })
    }
}

impl BlockGroupRenderState {
    pub fn bind_group_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
        /*
            @group(1) @binding(0) var<uniform> block_group_uniforms: BlockClusterUniforms;
            @group(1) @binding(1) var<storage, read> block_group_data: array<u32>;
        */
        let block_group_uniforms_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let block_group_data_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Block Bind Group Layout"),
                entries: &[block_group_uniforms_entry, block_group_data_entry],
            })
    }
}
