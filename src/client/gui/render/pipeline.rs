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
    include_wgsl, BindGroupLayout, BindGroupLayoutEntry, Device, PipelineLayoutDescriptor,
    PrimitiveState, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
};

pub struct BlockGroupPipeline {
    pub(crate) render_pipeline: RenderPipeline,
    // TODO: Move this to RenderClient!
    pub(crate) universe_bind_group_layout: BindGroupLayout,
    pub(crate) block_cluster_bind_group_layout: BindGroupLayout,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub view_matrix: [f32; 16],
    pub global_time: f32,
    pub _padding: UVec3,
}

impl BlockGroupPipeline {
    pub fn new(gpu_device: &Device, render_target_format: TextureFormat) -> Self {
        let shader = gpu_device.create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex = wgpu::ShaderStages::VERTEX;
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let both = wgpu::ShaderStages::VERTEX_FRAGMENT;

        /*
            @group(0) @binding(0) var material_atlas: texture_2d<f32>;
            @group(0) @binding(1) var material_atlas_s: sampler;
        */
        let material_atlas = BindGroupLayoutEntry {
            binding: 0,
            visibility: fragment,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let material_atlas_sampler = BindGroupLayoutEntry {
            binding: 1,
            visibility: fragment,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let global_bind_group_layout =
            gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Universe Bind Group Layout"),
                entries: &[material_atlas, material_atlas_sampler],
            });

        /*
            @group(1) @binding(0) var<uniform> globals: GlobalUniforms;
            @group(1) @binding(1) var<storage, read> block_definitions: array<BlockDefinition>;
            @group(1) @binding(2) var<uniform> block_group_uniforms: BlockClusterUniforms;
            @group(1) @binding(3) var<storage, read> block_group_data: array<u32>;
        */
        let global_uniforms = BindGroupLayoutEntry {
            binding: 0,
            visibility: vertex,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }; // TODO: Move to a different bind group.
        let block_group_palette_buffer = BindGroupLayoutEntry {
            binding: 1,
            visibility: both,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let block_group_uniforms = BindGroupLayoutEntry {
            binding: 2,
            visibility: vertex,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let block_group_data = BindGroupLayoutEntry {
            binding: 3,
            visibility: vertex,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let block_cluster_bind_group_layout =
            gpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Block Bind Group Layout"),
                entries: &[
                    global_uniforms,
                    block_group_palette_buffer,
                    block_group_uniforms,
                    block_group_data,
                ],
            });

        let render_pipeline_layout = gpu_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout Descriptor"),
            bind_group_layouts: &[
                Some(&global_bind_group_layout),
                Some(&block_cluster_bind_group_layout),
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
            render_pipeline,
            universe_bind_group_layout: global_bind_group_layout,
            block_cluster_bind_group_layout,
        }
    }
}
