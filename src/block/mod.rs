use glam::{Affine3, UVec3, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDefinition {
    pub display_name: String,
    pub material: PerFace<RenderMaterial>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMaterial {
    pub color: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderMaterialGpu {
    pub color: Vec3,
}

pub type BlockId = u32;

/// A Block entry in a Cluster.
#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub id: BlockId,
}

/// A collection of blocks that's somewhere in the world.
#[derive(Debug, Clone)]
pub struct BlockCluster {
    pub transform: Affine3,
    pub physics_mode: PhysicsMode,
    pub size: UVec3,
    pub blocks: Vec<Block>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockClusterGpuUniforms {
    pub transform: Affine3,
    pub size: UVec3,
}

pub struct BlockClusterRenderResources {
    pub block_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub num_voxels: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsMode {
    Ghost,
    Static,
    Kinematic,
    Dynamic,
}
