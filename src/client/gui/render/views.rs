use glam::{UVec3, Vec2};
use image::{EncodableLayout, ImageBuffer, Rgba};
use wgpu::util::DeviceExt;

use crate::{block::Block, client::gui::render::Gpu, universe::Universe};

/// A [`RenderClient`]'s "view" of a universe,
/// that is, resources necessary for understanding what the palettized block group data represents.
pub struct UniverseRenderView {
    pub block_definition_appearances: Vec<RenderMaterialView>,
    pub block_definition_appearances_gpu: wgpu::Buffer,

    pub material_texture_atlas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub material_texture_atlas_gpu: wgpu::Texture,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl UniverseRenderView {
    pub fn new(gpu: &Gpu, universe: &Universe) -> Self {
        // TODO: Send more complex appearances to the GPU, yeah!
        // Right now we are only sending materials and assuming
        // every block uses that material all over.
        let block_definition_appearances: Vec<RenderMaterialView> = universe
            .block_definitions
            .iter()
            .map(|(_, x)| x.appearance.material.x_max.clone())
            .map(|x| RenderMaterialView {
                atlas_position: x.atlas_position,
                atlas_size: x.atlas_size,
            })
            .collect();

        let block_definition_appearances_gpu =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Block Definition Appearances Buffer"),
                    contents: bytemuck::cast_slice(&block_definition_appearances),
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

        /*
            @group(0) @binding(0) var material_atlas: texture_2d<f32>;
            @group(0) @binding(1) var material_atlas_s: sampler;
        */
        let texture_atlas_view_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let texture_atlas_sampler_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Universe Bind Group Layout"),
                    entries: &[texture_atlas_view_entry, texture_atlas_sampler_entry],
                });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Universe Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas_sampler),
                },
            ],
        });

        UniverseRenderView {
            block_definition_appearances,
            block_definition_appearances_gpu,
            material_texture_atlas: texture_atlas,
            material_texture_atlas_gpu: texture_atlas_gpu,
            bind_group_layout,
            bind_group,
        }
    }
}

pub struct WorldRenderView {}

pub struct SceneRenderView {
    // TODO: Use a more efficient data structure.
    pub block_groups: Vec<BlockGroupRenderView>,
}

/// A render client's view of a block group.
///
/// This contains only its "appearance" and has no information about, say, its collision.
pub struct BlockGroupRenderView {
    pub uniforms: BlockGroupUniforms,
    pub uniforms_gpu: wgpu::Buffer,

    /// The voxel data inside of a this block group.
    /// Its indexes into a palette stored in the Universe (though perhaps it should be per-World).
    ///
    /// TODO: Abstract the data structure away (instead of using Vec).
    pub block_data: Vec<Block>,
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
