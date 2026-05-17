use crate::resources::material::MaterialDefinition;
use crate::resources::{
    block_type::BlockTypeDefinition, extract_all_from_folder, texture::TextureDefinition,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::PathBuf;

/// A collection of resources used to build worlds with.
#[derive(Clone, Debug)]
pub struct DataPack {
    pub id: String,
    pub block_definitions: IndexMap<String, BlockTypeDefinition>,
    pub material_definitions: IndexMap<String, MaterialDefinition>,
    pub texture_definitions: IndexMap<String, TextureDefinition>,
}

crate::impl_id!(DataPack);

/// File on disk that defines a folder as containing a data pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPackDefinition {
    id: String,
}

crate::impl_id!(DataPackDefinition);

const DATA_PACK_MANIFEST_NAME: &str = "pack.ron";
const DATA_PACK_BLOCKS_DIR_NAME: &str = "blocks";
const _DATA_PACK_APPEARANCES_DIR_NAME: &str = "appearances";
const DATA_PACK_MATERIALS_DIR_NAME: &str = "materials";
const DATA_PACK_TEXTURES_DIR_NAME: &str = "textures";

impl DataPack {
    pub fn from_directory(data_path_dir_path: PathBuf) -> std::io::Result<Self> {
        let manifest = std::fs::read_to_string(data_path_dir_path.join(DATA_PACK_MANIFEST_NAME))?;
        let manifest: DataPackDefinition =
            ron::from_str(manifest.as_str()).expect("Failed to parse manifest...");

        let mut pack = DataPack {
            id: manifest.id,
            block_definitions: IndexMap::new(),
            material_definitions: IndexMap::new(),
            texture_definitions: IndexMap::new(),
        };

        extract_all_from_folder(
            &mut pack.texture_definitions,
            data_path_dir_path.join(DATA_PACK_TEXTURES_DIR_NAME),
        )?;

        extract_all_from_folder(
            &mut pack.material_definitions,
            data_path_dir_path.join(DATA_PACK_MATERIALS_DIR_NAME),
        )?;

        extract_all_from_folder(
            &mut pack.block_definitions,
            data_path_dir_path.join(DATA_PACK_BLOCKS_DIR_NAME),
        )?;

        Ok(pack)
    }
}

impl PartialEq for DataPack {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DataPack {}

impl Hash for DataPack {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
