use glam::Vec2;
use indexmap::IndexMap;

use crate::{
    data_packs::Universe,
    world::block::{BlockAppearance, BlockDefinition, PerFace, RenderMaterial},
};

impl Universe {
    pub fn example() -> Self {
        let mut block_definitions: IndexMap<String, BlockDefinition> = IndexMap::new();

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

    pub fn block_definitions(&self) -> Vec<BlockDefinition> {
        self.block_definitions
            .iter()
            .map(|(_, bd)| bd.clone())
            .collect::<Vec<_>>()
    }
}
