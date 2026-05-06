use self::pipeline::voxels::WorldUniforms;

use super::view::{BlockTypeId, LayoutId, LayoutRef};
use dashmap::DashMap;
use glam::{UVec3, Vec2};
use image::{ImageBuffer, Rgba};

pub mod gpu;
pub mod pipeline;
pub mod render_target;

/// Module for progressive modifications to render states.
pub mod patch;

pub struct GameRenderState {
    pub universe_state: UniverseRenderState,
    pub world_state: WorldRenderState,
}

pub struct UniverseRenderState {
    pub block_appearance_palette: DashMap<BlockTypeId, BlockAppearanceEntry>,
    pub block_appearance_palette_gpu: wgpu::Buffer,

    pub texture_atlas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub texture_atlas_gpu: wgpu::Texture,

    pub bind_group: wgpu::BindGroup,
}

pub struct WorldRenderState {
    pub current_scene: CurrentSceneRenderState,
    pub layout_cache: DashMap<LayoutId, LayoutRenderState>,

    pub uniforms: WorldUniforms,
    pub uniforms_gpu: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub struct CurrentSceneRenderState {
    pub environment: EnvironmentRenderState,
    pub root_layout: LayoutRenderState,
}

pub struct EnvironmentRenderState {
    // Nothing for now.
}

pub struct LayoutRenderState {
    // TODO: I think this might require a different data structure?
    // possibly an Arena?
    pub block_groups: Vec<BlockGroupRenderState>,
    pub sub_layouts: Vec<LayoutRef>,
    // TODO: Add transient data!
}

pub struct BlockGroupRenderState {
    // TODO: Maybe store uniform data for all block groups in a single giant buffer!
    pub uniforms: BlockGroupUniforms,
    pub uniforms_gpu: wgpu::Buffer,

    // TODO: Maybe store block group data for non-dynamic block groups in a single giant buffer.
    // Then draw them all with indirect drawing perhaps!
    pub block_appearance_data: Vec<BlockAppearanceEntry>,
    pub block_appearance_data_gpu: wgpu::Buffer,

    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockGroupUniforms {
    pub transform: [f32; 16],
    pub size: UVec3,
    pub _padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockAppearanceEntry {
    pub idx_in_palette: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureRect {
    atlas_position: Vec2,
    atlas_size: Vec2,
}
