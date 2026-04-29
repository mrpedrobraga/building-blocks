//! # Blocks Pipeline
//!
//! The pipeline used by the default client to render scenes!
//!
//! ## What's a scene made of?
//!
//! A scene is made of an assortment of visible things which are rendered in batches.
//!
//! 1. First are rendered the block clusters — those are 3D nametables indexing into a block palette.
//!     1.1. Collections of voxels can be rendered all at once in a single draw call!
//!     1.2. Specialized voxels with complex, non-cuboid models will be drawn all together in a single call, too.
//! 2. Second are rendered the sprites — those are independent models that have arbitrary positions.
//!     2.1. Sprites are made of regular geometry like boxes and planes, so they can be all drawn in a single call.
//!
//! ## The Shader
//!
//!

use glam::UVec3;
use wgpu::{
    include_wgsl, BindGroupLayout, Device, PipelineLayoutDescriptor, PrimitiveState,
    RenderPipeline, RenderPipelineDescriptor, TextureFormat,
};

pub struct BlockGroupPipeline {
    pub universe_bind_group_layout: BindGroupLayout,
    pub block_group_bind_group_layout: BindGroupLayout,
    pub render_pipeline: RenderPipeline,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub view_matrix: [f32; 16],
    pub global_time: f32,
    pub _padding: [f32; 3],
}

impl BlockGroupPipeline {
    pub fn new(
        gpu_device: &Device,
        render_target_format: TextureFormat,
        universe_bind_group_layout: wgpu::BindGroupLayout,
        block_group_bind_group_layout: wgpu::BindGroupLayout,
    ) -> Self {
        let shader = gpu_device.create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = gpu_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout Descriptor"),
            bind_group_layouts: &[
                Some(&universe_bind_group_layout),
                Some(&block_group_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let render_pipeline = gpu_device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        BlockGroupPipeline {
            universe_bind_group_layout,
            block_group_bind_group_layout,
            render_pipeline,
        }
    }
}
