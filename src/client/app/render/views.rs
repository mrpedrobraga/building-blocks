use glam::{Mat4, UVec3, Vec2};
use image::{EncodableLayout, ImageBuffer, Rgba};
use wgpu::{util::DeviceExt, BindGroupLayoutEntry};

use crate::{
    client::app::render::Gpu,
    data_packs::Universe,
    world::{block::BlockGroup, Scene},
};

/// A [`RenderClient`]'s "view" of a universe,
/// that is, resources necessary for understanding what the palettized block group data represents.
pub struct UniverseRenderView {
    pub block_definition_appearances: Vec<RenderMaterialView>,
    pub block_definition_appearances_gpu: wgpu::Buffer,

    pub material_texture_atlas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub material_texture_atlas_gpu: wgpu::Texture,

    pub bind_group: wgpu::BindGroup,
}

impl UniverseRenderView {
    pub fn new(
        gpu: &Gpu,
        uniform_buffer: &wgpu::Buffer,
        universe: &Universe,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // TODO: Send more complex appearances to the GPU, yeah!
        // Right now we are only sending materials and assuming
        // every block uses that material all over.
        let textures = universe.textures_cloned();
        let materials = universe.materials_cloned();

        let block_definition_materials: Vec<RenderMaterialView> = universe
            .block_types()
            .iter()
            .filter_map(|block_type| match &block_type.appearance {
                // TODO: Properly support these cuboid materials!
                crate::models::BlockAppearance::Cuboid { x_min, .. } => Some(x_min),
            })
            .map(|material_ref| {
                materials
                    .iter()
                    .find(|material| material.id == material_ref.id)
            })
            .filter_map(|x| x)
            .map(|material| {
                let rect = material
                    .albedo
                    .rect
                    .as_ref()
                    .expect("No rect provided for texture ref?");
                // TODO: Use the texture as well as the texture size
                // in the rendermaterialview information.
                let _texture = textures.iter().find(|tex| tex.id == material.albedo.id);

                RenderMaterialView {
                    // TODO: Texture index since more than one texture can be provided.
                    atlas_position: rect.position,
                    atlas_size: rect.size,
                }
            })
            .collect();

        let block_definition_appearances_gpu =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Block Definition Appearances Buffer"),
                    contents: bytemuck::cast_slice(&block_definition_materials),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        // TODO: Load the atlases from the universe...
        // Maybe put them all in an ArrayTexture!
        let texture_atlas = image::open("src/client/gui/render/debug_atlas.png")
            .unwrap()
            .to_rgba8();
        let texture_atlas_descriptor = wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: texture_atlas.width(),
                height: texture_atlas.height(),
                // TODO: Yeah this here would allow us to
                // have multiple atlases :-)
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture_atlas_gpu = gpu.device.create_texture_with_data(
            &gpu.queue,
            &texture_atlas_descriptor,
            wgpu::wgt::TextureDataOrder::LayerMajor,
            texture_atlas.as_bytes(),
        );

        let texture_atlas_view = texture_atlas_gpu.create_view(&Default::default());
        let texture_atlas_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Universe Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: block_definition_appearances_gpu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas_sampler),
                },
            ],
        });

        UniverseRenderView {
            block_definition_appearances: block_definition_materials,
            block_definition_appearances_gpu,
            material_texture_atlas: texture_atlas,
            material_texture_atlas_gpu: texture_atlas_gpu,
            bind_group,
        }
    }

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

pub struct WorldRenderView {}

pub struct SceneRenderView {
    // TODO: Use a more efficient data structure.
    pub block_groups: Vec<BlockGroupRenderView>,
}

impl SceneRenderView {
    pub fn new(gpu: &Gpu, scene: &Scene, block_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let block_groups: Vec<_> = scene
            .root_layout
            .block_groups
            .iter()
            .map(|(_, block_group)| {
                BlockGroupRenderView::new(gpu, &block_group, block_bind_group_layout)
            })
            .collect();

        Self { block_groups }
    }
}

/// A render client's view of a block group.
///
/// This contains only its "appearance" and has no information about, say, its collision.
pub struct BlockGroupRenderView {
    pub uniforms: BlockGroupUniforms,
    /// TODO: Instead of one buffer per block group, we could aggregate
    /// data from several [`BlockGroupRenderView`]s to one single buffer with offsets.
    /// This buffer should be stored possibly per layout or per scene.
    pub uniforms_gpu: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,

    /// The voxel data inside of a this block group.
    /// Its indexes into a palette stored in the Universe (though perhaps it should be per-World).
    ///
    /// TODO: Abstract the data structure away (instead of using Vec).
    pub block_data: Vec<BlockView>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockView {
    pub idx_in_palette: u32,
}

impl BlockGroupRenderView {
    pub fn volume(&self) -> u32 {
        self.uniforms.size.x * self.uniforms.size.y * self.uniforms.size.z
    }

    pub fn new(gpu: &Gpu, block_group: &BlockGroup, layout: &wgpu::BindGroupLayout) -> Self {
        let uniforms = BlockGroupUniforms {
            transform: Mat4::from_mat3a(block_group.transform.matrix3).to_cols_array(),
            size: block_group.size,
            _padding: 0,
        };
        let uniforms_gpu = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Block Group Uniforms Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let block_data: Vec<_> = block_group
            .blocks
            .iter()
            .map(|b| BlockView {
                idx_in_palette: b.idx_in_palette,
            })
            .collect();
        let block_data_gpu = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Block Group Data"),
                contents: bytemuck::cast_slice(&block_data),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Block Group Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_gpu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: block_data_gpu.as_entire_binding(),
                },
            ],
        });

        Self {
            uniforms,
            uniforms_gpu,
            block_data,
            bind_group,
        }
    }

    pub fn bind_group_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
        /*
            @group(1) @binding(0) var<uniform> block_group_uniforms: BlockClusterUniforms;
            @group(1) @binding(1) var<storage, read> block_group_data: array<u32>;
        */
        let block_group_uniforms_entry = BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let block_group_data_entry = BindGroupLayoutEntry {
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockGroupUniforms {
    pub transform: [f32; 16],
    pub size: UVec3,
    pub _padding: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderMaterialView {
    pub atlas_position: Vec2,
    pub atlas_size: Vec2,
}
