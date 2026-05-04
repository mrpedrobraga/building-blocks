use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::data_packs::{DataPack, Universe};
use std::{ffi::OsStr, path::PathBuf};

pub trait Identifiable {
    fn id(&self) -> String;
}

/// File on disk that defines a folder as containing a universe!
#[derive(Debug, Serialize, Deserialize)]
pub struct UniverseDefinition {
    pub id: String,
    pub display_name: String,
    pub data_packs: Vec<UniversePackRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UniversePackRef {
    /// The pack is stored locally on a folder.
    Local(PathBuf),
    /// The pack is sourced remotely. It might be cached!
    Git(String),
}

const UNIVERSE_MANIFEST_NAME: &'static str = "universe.ron";

impl Identifiable for UniverseDefinition {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Universe {
    pub fn from_directory(universe_dir_path: PathBuf) -> std::io::Result<Self> {
        let manifest = std::fs::read_to_string(universe_dir_path.join(UNIVERSE_MANIFEST_NAME))?;
        let manifest: UniverseDefinition =
            ron::from_str(manifest.as_str()).expect("Failed to parse manifest...");

        let mut data_packs = IndexSet::new();

        for pack in manifest.data_packs {
            match pack {
                UniversePackRef::Local(data_pack_dir_path) => {
                    let data_pack =
                        DataPack::from_directory(universe_dir_path.join(data_pack_dir_path))?;
                    data_packs.insert(data_pack);
                }
                UniversePackRef::Git(_) => {
                    warn!("Remote data packs are not yet implemented.")
                }
            }
        }

        let universe = Universe {
            id: manifest.id,
            data_packs,
        };

        Ok(universe)
    }
}

/// File on disk that defines a folder as containing a data pack.
#[derive(Debug, Serialize, Deserialize)]
pub struct DataPackDefinition {
    id: String,
}

const DATA_PACK_MANIFEST_NAME: &'static str = "pack.ron";
const DATA_PACK_BLOCKS_DIR_NAME: &'static str = "blocks";
const DATA_PACK_APPEARANCES_DIR_NAME: &'static str = "appearances";
const DATA_PACK_MATERIALS_DIR_NAME: &'static str = "materials";
const DATA_PACK_TEXTURES_DIR_NAME: &'static str = "textures";

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

fn extract_all_from_folder<T: 'static + Identifiable + for<'de> Deserialize<'de>>(
    target: &mut IndexMap<String, T>,
    dir: PathBuf,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.extension() == Some(&OsStr::new("ron")) {
            let raw_block_definition = std::fs::read_to_string(entry.path())?;
            println!("\n\nReading {}\n", raw_block_definition);
            let block_definition: T =
                ron::from_str(raw_block_definition.as_str()).expect("Failed to parse ron file.");
            target.insert(block_definition.id(), block_definition);
        }
    }

    Ok(())
}

#[test]
fn test_u_loadin() {
    let u = Universe::from_directory(PathBuf::from("./examples/example_universe")).unwrap();
    dbg!(u);
}
