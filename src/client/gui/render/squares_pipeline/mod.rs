use glam::UVec2;
use image::Rgba;
use wgpu::{include_wgsl, Device, PipelineLayout, PrimitiveState, RenderPipeline, TextureFormat};

pub struct SquaresPipeline {
    pub global_uniforms: GlobalUniforms,
    pub layers: Vec<Layer>,

    pub layers_to_cache_pipeline_layout: PipelineLayout,
    pub layers_to_cache_pipeline: RenderPipeline,
    pub main_texture: wgpu::Texture,

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
    pub fn new(
        gpu_device: &Device,
        render_target_format: TextureFormat,
        render_target_size: UVec2,
    ) -> Self {
        let global_uniforms = GlobalUniforms {};
        let layers = vec![];
        let main_texture = gpu_device.create_texture(&wgpu::TextureDescriptor {
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
        });

        let shader = gpu_device.create_shader_module(include_wgsl!("shader.wgsl"));

        let layers_to_cache_pipeline_layout =
            gpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline"),
                // TODO: Get some bind groups here!
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let layers_to_cache_pipeline =
            gpu_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        let layer_to_screen_shader =
            gpu_device.create_shader_module(include_wgsl!("layers_to_screen.wgsl"));

        let render_to_screen_pipeline_layout =
            gpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Layers to Screen Pipeline Layout"),
                // TODO: Get some bind groups here!
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let render_to_screen_pipeline =
            gpu_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Layers to Screen Pipeline"),
                layout: Some(&layers_to_cache_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &layer_to_screen_shader,
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
                    module: &layer_to_screen_shader,
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
            render_to_screen_pipeline_layout,
            render_to_screen_pipeline,
            main_texture,
        }
    }
}
