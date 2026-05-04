use glam::Vec2;
use indexmap::{indexset, IndexMap};

use crate::{
    resources::{
        block_type::{BlockAppearance, BlockTypeDefinition},
        data_pack::DataPack,
        material::{MaterialDefinition, MaterialRef},
        texture::{Rect, TextureDefinition, TextureRef},
        universe::Universe,
    },
    world::block::BlockType,
};

impl Universe {
    pub fn example() -> Self {
        let texture_definitions: IndexMap<String, TextureDefinition> = IndexMap::new();

        let mut material_definitions: IndexMap<String, MaterialDefinition> = IndexMap::new();

        let dirt_material = MaterialDefinition {
            id: "toyvox:dirt".to_string(),
            display_name: "Dirt".to_string(),
            albedo: TextureRef {
                id: "toyvox:main-atlas".to_string(),
                rect: Some(Rect {
                    position: Vec2::new(0.0, 0.0),
                    size: Vec2::new(8.0, 8.0),
                }),
            },
        };

        let grassy_dirt_material = MaterialDefinition {
            id: "toyvox:grassy_dirt".to_string(),
            display_name: "Grassy Dirt".to_string(),
            albedo: TextureRef {
                id: "toyvox:main-atlas".to_string(),
                rect: Some(Rect {
                    position: Vec2::new(8.0, 0.0),
                    size: Vec2::new(8.0, 8.0),
                }),
            },
        };

        material_definitions.insert("toyvox:dirt".to_string(), dirt_material);
        material_definitions.insert("toyvox:grassy_dirt".to_string(), grassy_dirt_material);

        let mut block_types: IndexMap<String, BlockTypeDefinition> = IndexMap::new();

        block_types.insert(
            "dirt".to_string(),
            BlockTypeDefinition {
                id: "toyvox:dirt".to_string(),
                display_name: "Dirt".to_string(),
                appearance: BlockAppearance::Cuboid {
                    x_min: MaterialRef {
                        id: String::from("toyvox:dirt"),
                    },
                    x_max: MaterialRef {
                        id: String::from("toyvox:dirt"),
                    },
                    y_min: MaterialRef {
                        id: String::from("toyvox:dirt"),
                    },
                    y_max: MaterialRef {
                        id: String::from("toyvox:dirt"),
                    },
                    z_min: MaterialRef {
                        id: String::from("toyvox:dirt"),
                    },
                    z_max: MaterialRef {
                        id: String::from("toyvox:dirt"),
                    },
                },
            },
        );

        block_types.insert(
            "grassy_dirt".to_string(),
            BlockTypeDefinition {
                id: "toyvox:grassy_dirt".to_string(),
                display_name: "Grassy Dirt".to_string(),
                appearance: BlockAppearance::Cuboid {
                    x_min: MaterialRef {
                        id: String::from("toyvox:grassy_dirt"),
                    },
                    x_max: MaterialRef {
                        id: String::from("toyvox:grassy_dirt"),
                    },
                    y_min: MaterialRef {
                        id: String::from("toyvox:grassy_dirt"),
                    },
                    y_max: MaterialRef {
                        id: String::from("toyvox:grassy_dirt"),
                    },
                    z_min: MaterialRef {
                        id: String::from("toyvox:grassy_dirt"),
                    },
                    z_max: MaterialRef {
                        id: String::from("toyvox:grassy_dirt"),
                    },
                },
            },
        );

        let data_pack = DataPack {
            id: String::from("default"),
            block_definitions: block_types,
            material_definitions,
            texture_definitions,
        };

        let data_packs = indexset! { data_pack };

        Universe {
            id: String::from("example"),
            data_packs,
        }
    }

    pub fn block_types(&self) -> Vec<BlockType> {
        self.data_packs
            .iter()
            .flat_map(|d| {
                d.block_definitions
                    .iter()
                    .map(|(_, block_type_def)| BlockType::from_definition(block_type_def))
            })
            .collect::<Vec<_>>()
    }

    pub fn material(&self, id: String) -> Option<&MaterialDefinition> {
        for data_pack in self.data_packs.iter().rev() {
            if let Some(material) = data_pack.material_definitions.get(&id) {
                return Some(material);
            }
        }

        None
    }

    pub fn materials_cloned(&self) -> Vec<MaterialDefinition> {
        self.data_packs
            .iter()
            .flat_map(|d| {
                d.material_definitions
                    .iter()
                    .map(|(_, material)| material.clone())
            })
            .collect()
    }

    pub fn texture(&self, id: String) -> Option<&TextureDefinition> {
        for data_pack in self.data_packs.iter().rev() {
            if let Some(texture) = data_pack.texture_definitions.get(&id) {
                return Some(texture);
            }
        }

        None
    }

    pub fn textures_cloned(&self) -> Vec<TextureDefinition> {
        self.data_packs
            .iter()
            .flat_map(|d| {
                d.texture_definitions
                    .iter()
                    .map(|(_, material)| material.clone())
            })
            .collect()
    }
}
