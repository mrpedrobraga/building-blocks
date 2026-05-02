use generational_arena::Arena;
use glam::{Affine3A, UVec3, Vec3};
use indexmap::indexmap;

use crate::{
    world::block::{Block, BlockGroup},
    world::{Environment, Layout, Scene, World},
};

impl World {
    pub fn example() -> Self {
        let block_group_0 = BlockGroup {
            transform: Affine3A::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            physics_mode: crate::world::block::PhysicsMode::Dynamic,
            size: UVec3::new(3, 3, 3),
            blocks: example_blocks(),
        };

        let mut block_groups = Arena::new();
        block_groups.insert(block_group_0);

        let scene = Scene {
            environment: Environment {
                xy_plane_is_solid_floor: true,
            },
            root_layout: Layout { block_groups },
        };

        World {
            id: "example".to_string(),
            scenes: indexmap! {
                String::from("default") => scene
            },
        }
    }
}

fn example_blocks() -> Vec<Block> {
    const AIR: u32 = 0;
    //const DIRT: u32 = 1;
    const WOOD: u32 = 2;

    vec![
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        // ---
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        // ---
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: AIR,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
        Block {
            idx_in_palette: WOOD,
        },
    ]
}
