//! # Universes
//!
//! Since the engine is data-driven, a Universe represents a set of
//! definitions — all the existing blocks and items, systems, etc.

use generational_arena::Arena;
use glam::{Affine3A, UVec3, Vec2, Vec3};
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
    pub material_texture_atlas: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

/// A World contains blocks, clusters and entities.
pub struct World {
    pub block_clusters: Arena<BlockCluster>,
}

impl Universe {
    pub fn example() -> Self {
        let mut block_definitions: IndexMap<Id, BlockDefinition> = IndexMap::new();

        block_definitions.insert(
            "dirt".to_string(),
            BlockDefinition {
                display_name: "Dirt".to_string(),
                material: PerFace::homogeneous(RenderMaterial {
                    atlas_position: Vec2::new(0.0, 0.0),
                    atlas_size: Vec2::new(8.0, 8.0),
                }),
            },
        );

        block_definitions.insert(
            "grassy_dirt".to_string(),
            BlockDefinition {
                display_name: "Grassy Dirt".to_string(),
                material: PerFace::homogeneous(RenderMaterial {
                    atlas_position: Vec2::new(8.0, 0.0),
                    atlas_size: Vec2::new(8.0, 8.0),
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
        let size = UVec3::new(3, 3, 3);

        const AIR: u32 = 0;
        //const DIRT: u32 = 1;
        const WOOD: u32 = 2;

        let blocks = vec![
            Block { id: WOOD },
            Block { id: AIR },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: AIR },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: WOOD },
            // ---
            Block { id: AIR },
            Block { id: AIR },
            Block { id: AIR },
            Block { id: AIR },
            Block { id: AIR },
            Block { id: AIR },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: WOOD },
            // ---
            Block { id: WOOD },
            Block { id: AIR },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: AIR },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: WOOD },
            Block { id: WOOD },
        ];

        let example_block_cluster = BlockCluster {
            transform: Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            physics_mode: crate::block::PhysicsMode::Dynamic,
            size,
            blocks,
            gpu: None,
        };

        let mut block_clusters = Arena::new();
        block_clusters.insert(example_block_cluster);

        World { block_clusters }
    }
}
