//! # Universes
//!
//! Since the engine is data-driven, a Universe represents a set of
//! definitions — all the existing blocks and items, systems, etc.

use generational_arena::Arena;
use glam::{Affine3A, UVec3, Vec3, Vec4};
use indexmap::IndexMap;

use crate::{
    block::{Block, BlockCluster, BlockDefinition, PerFace, RenderMaterial},
    Id,
};

/// See the module-level documentation.
pub struct Universe {
    pub block_definitions: IndexMap<Id, BlockDefinition>,
    pub gpu: Option<UniverseGpu>,
}

pub struct UniverseGpu {
    pub block_definitions_buffer: wgpu::Buffer,
}

/// A World contains blocks, clusters and entities.
pub struct World {
    pub block_clusters: Arena<BlockCluster>,
}

impl Universe {
    pub fn example() -> Self {
        let mut block_definitions: IndexMap<Id, BlockDefinition> = IndexMap::new();

        block_definitions.insert(
            "cyan".to_string(),
            BlockDefinition {
                display_name: "Cyan Block".to_string(),
                material: PerFace::homogeneous(RenderMaterial {
                    color: Vec4::new(0.0, 1.0, 1.0, 1.0),
                }),
            },
        );

        block_definitions.insert(
            "red".to_string(),
            BlockDefinition {
                display_name: "Red Block".to_string(),
                material: PerFace::homogeneous(RenderMaterial {
                    color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                }),
            },
        );

        Universe {
            block_definitions,
            gpu: None,
        }
    }
}

impl World {
    pub fn example() -> Self {
        let example_block_cluster = BlockCluster {
            transform: Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            physics_mode: crate::block::PhysicsMode::Dynamic,
            size: UVec3::new(2, 1, 1),
            blocks: vec![Block { id: 1 }, Block { id: 2 }],
            gpu: None,
        };

        let mut block_clusters = Arena::new();
        block_clusters.insert(example_block_cluster);

        World { block_clusters }
    }
}
