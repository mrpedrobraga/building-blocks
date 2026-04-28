//! # Universes
//!
//! Since the engine is data-driven, a Universe represents a set of
//! definitions — all the existing blocks and items, systems, etc.

use generational_arena::Arena;
use glam::{Affine3A, UVec3, Vec2, Vec3};
use indexmap::IndexMap;

use crate::{
    block::{Block, BlockAppearance, BlockDefinition, BlockGroup, PerFace, RenderMaterial},
    Id,
};

/// See the module-level documentation.
#[derive(Clone)]
pub struct Universe {
    pub block_definitions: IndexMap<Id, BlockDefinition>,
}

/// A World is a collection of scenes plus some metadata.
pub struct World {
    pub scenes: IndexMap<Id, Scene>,
}

/// A scene is a root layout plus some environment data!
#[derive(Clone)]
pub struct Scene {
    pub environment: Environment,
    pub root_layout: Layout,
}

/// The environment of a scene.
///
/// Futurely, this will contain per scene metadata such as
/// the skybox, lighting, which shaders to use, etc.
#[derive(Clone)]
pub struct Environment {}

/// A layout is a reusable collection of block groups, entities, etc.
#[derive(Clone)]
pub struct Layout {
    pub block_groups: Arena<BlockGroup>,
}

impl Universe {
    pub fn example() -> Self {
        let mut block_definitions: IndexMap<Id, BlockDefinition> = IndexMap::new();

        block_definitions.insert(
            "dirt".to_string(),
            BlockDefinition {
                display_name: "Dirt".to_string(),
                appearance: BlockAppearance {
                    material: PerFace::homogeneous(RenderMaterial {
                        atlas_position: Vec2::new(0.0, 0.0),
                        atlas_size: Vec2::new(8.0, 8.0),
                    }),
                },
            },
        );

        block_definitions.insert(
            "grassy_dirt".to_string(),
            BlockDefinition {
                display_name: "Grassy Dirt".to_string(),
                appearance: BlockAppearance {
                    material: PerFace::homogeneous(RenderMaterial {
                        atlas_position: Vec2::new(8.0, 0.0),
                        atlas_size: Vec2::new(8.0, 8.0),
                    }),
                },
            },
        );

        Universe { block_definitions }
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

        let block_group_0 = BlockGroup {
            transform: Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            physics_mode: crate::block::PhysicsMode::Dynamic,
            size,
            blocks,
        };

        let mut block_groups = Arena::new();
        block_groups.insert(block_group_0);

        let root_layout = Layout { block_groups };

        let scene = Scene {
            environment: Environment {},
            root_layout,
        };

        let mut scenes = IndexMap::new();
        scenes.insert("default".to_owned(), scene);

        World { scenes }
    }
}
