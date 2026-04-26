use glam::{Affine3A, UVec3, UVec4, Vec4};
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
    pub color: Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderMaterialGpu {
    pub color: Vec4,
}

pub type BlockId = u32;

/// A Block entry in a Cluster.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
}

/// A collection of blocks that's somewhere in the world.
#[derive(Serialize, Deserialize)]
pub struct BlockCluster {
    pub transform: Affine3A,
    pub physics_mode: PhysicsMode,
    pub size: UVec3,
    pub blocks: Vec<Block>,

    #[serde(skip_serializing, skip_deserializing)]
    pub gpu: Option<(BlockClusterGpuUniforms, BlockClusterRenderResources)>,
}

impl Clone for BlockCluster {
    fn clone(&self) -> Self {
        Self {
            transform: self.transform.clone(),
            physics_mode: self.physics_mode.clone(),
            size: self.size.clone(),
            blocks: self.blocks.clone(),
            gpu: None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockClusterGpuUniforms {
    // TODO: Move this out of cluster uniforms and into
    // a camera uniform :-)
    pub view_projection: [f32; 16],
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
