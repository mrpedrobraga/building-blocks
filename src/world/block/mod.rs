//! # Blocks
//!
//! Blocks are the building... uh... well, the basic elements of creation in the world.
//!
//! The data format the engine uses works like this:
//!
//! - There are [`BlockDefinition`]s, which define what a block is, how it looks like, etc.
//! - There are [`BlockGroup`]s, which are like Minecraft chunks, collections of data. Actually, they are more like Sable [sub-levels](https://modrinth.com/mod/sable), since they are able to have arbitrary size, rotation, and physics.
//!
//! Within BlockGroups, a block is represented by [`Block`], which contains an index into a global block palette. This makes storage really cheap.
//! Since a Block is a 32-bit unsigned integer, this means you can have up to `4_294_967_296` different block types. This format will change soon in order to support more advanced kinds of blocks (such as proxy blocks!);

use glam::{Affine3A, UVec3, UVec4, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    resources::block_type::{BlockAppearance, BlockTypeDefinition},
    resources::Id,
};

/// Information about what a given block "is":
/// Its appearance, general information and physics;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockType {
    pub id: String,
    pub display_name: String,
    pub appearance: BlockAppearance,
}

impl Id for BlockType {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl BlockType {
    pub fn from_definition(definition: &BlockTypeDefinition) -> Self {
        Self {
            id: definition.id.clone(),
            display_name: definition.display_name.clone(),
            appearance: definition.appearance.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerFace<T> {
    pub x_min: T,
    pub x_max: T,
    pub y_min: T,
    pub y_max: T,
    pub z_min: T,
    pub z_max: T,
}

impl<T> PerFace<T>
where
    T: Clone,
{
    pub fn homogeneous(value: T) -> Self {
        Self {
            x_min: value.clone(),
            x_max: value.clone(),
            y_min: value.clone(),
            y_max: value.clone(),
            z_min: value.clone(),
            z_max: value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMaterial {
    pub atlas_position: Vec2,
    pub atlas_size: Vec2,
}

/// A single entry in a [`BlockGroup`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Block {
    pub idx_in_palette: u32,
}

/// A collection of blocks that's somewhere in the world.
#[derive(Serialize, Deserialize)]
pub struct BlockGroup {
    pub transform: Affine3A,
    pub physics_mode: PhysicsMode,
    pub size: UVec3,
    pub blocks: Vec<Block>,
    // #[serde(skip_serializing, skip_deserializing)]
    // pub gpu: Option<(BlockClusterGpuUniforms, BlockClusterRenderResources)>,
}

impl Clone for BlockGroup {
    fn clone(&self) -> Self {
        Self {
            transform: self.transform.clone(),
            physics_mode: self.physics_mode.clone(),
            size: self.size.clone(),
            blocks: self.blocks.clone(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockClusterGpuUniforms {
    pub transform: [f32; 16],
    pub size: UVec4,
}

pub struct BlockClusterRenderResources {
    pub block_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub num_voxels: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsMode {
    Ghost,
    Static,
    Kinematic,
    Dynamic,
}
