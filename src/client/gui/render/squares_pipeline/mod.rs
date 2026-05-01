use glam::UVec2;
use image::{EncodableLayout, Rgba};
use wgpu::{
    include_wgsl, util::DeviceExt, PipelineLayout, PrimitiveState, RenderPipeline, TextureFormat,
};

use crate::client::gui::render::Gpu;

pub struct SquaresPipeline {
    pub global_uniforms: GlobalUniforms,
    pub layers: Vec<Layer>,

    pub layers_to_cache_pipeline_layout: PipelineLayout,
    pub layers_to_cache_pipeline: RenderPipeline,
    pub main_texture: wgpu::Texture,

    pub render_to_screen_bind_group: wgpu::BindGroup,

    pub render_to_screen_pipeline_layout: PipelineLayout,
    pub render_to_screen_pipeline: RenderPipeline,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {}

pub struct Layer {
    pub pixels: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub texture: wgpu::Texture,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LayerUniforms {
    pub transform: [f32; 16],
}

impl SquaresPipeline {
    pub fn new(gpu: &Gpu, render_target_format: TextureFormat, render_target_size: UVec2) -> Self {
        let global_uniforms = GlobalUniforms {};
        let layers = vec![];

        let example_gui_image =
            image::open("./src/client/gui/render/squares_pipeline/test_gui.png").unwrap();

        let main_texture = gpu.device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                label: Some("Screen Main Render Texture"),
                size: wgpu::Extent3d {
                    width: render_target_size.x,
                    height: render_target_size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: render_target_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            wgpu::wgt::TextureDataOrder::LayerMajor,
            example_gui_image.as_rgba8().unwrap().as_bytes(),
        );

        let shader = gpu
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let layers_to_cache_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline"),
                    // TODO: Get some bind groups here!
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });
        let layers_to_cache_pipeline =
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Screen Main Render Pipeline"),
                    layout: Some(&layers_to_cache_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    // TODO: Maybe add a depth stencil for GUI so it can interact with the world?
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: render_target_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    // TODO: Enable this in VR!
                    // You can imagine that GUI will always be rendered at a certain distance from the camera,
                    // even if it's further than 3D geometry it occludes!
                    multiview_mask: None,
                    cache: None,
                });

        let main_texture_view = main_texture.create_view(&Default::default());
        let main_texture_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Squares Main Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let render_to_screen_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Squares Render to Screen Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });
        let render_to_screen_bind_group =
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Squares Render to Screen Bind Group"),
                layout: &render_to_screen_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&main_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&main_texture_sampler),
                    },
                ],
            });

        let render_to_screen_shader = gpu
            .device
            .create_shader_module(include_wgsl!("layers_to_screen.wgsl"));

        let render_to_screen_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render to Screen Pipeline Layout"),
                    // TODO: Get some bind groups here!
                    bind_group_layouts: &[Some(&render_to_screen_bind_group_layout)],
                    immediate_size: 0,
                });
        let render_to_screen_pipeline =
            gpu.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render to Screen Pipeline"),
                    layout: Some(&render_to_screen_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &render_to_screen_shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    // TODO: Maybe add a depth stencil for GUI so it can interact with the world?
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
                        module: &render_to_screen_shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: render_target_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    // TODO: Enable this in VR!
                    // You can imagine that GUI will always be rendered at a certain distance from the camera,
                    // even if it's further than 3D geometry it occludes!
                    multiview_mask: None,
                    cache: None,
                });

        SquaresPipeline {
            global_uniforms,
            layers,
            layers_to_cache_pipeline_layout,
            layers_to_cache_pipeline,
            render_to_screen_bind_group,
            render_to_screen_pipeline_layout,
            render_to_screen_pipeline,
            main_texture,
        }
    }
}
