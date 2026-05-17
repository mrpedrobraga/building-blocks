use crate::resources::data_pack::DataPack;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::warn;

/// See the module-level documentation.
#[derive(Clone, Debug)]
pub struct Universe {
    pub id: String,
    pub data_packs: IndexSet<DataPack>,
}

/// File on disk that defines a folder as containing a universe!
#[derive(Debug, Serialize, Deserialize)]
pub struct UniverseDefinition {
    pub id: String,
    pub display_name: String,
    pub data_packs: Vec<DataPackRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataPackRef {
    /// The pack is stored locally on a folder.
    Local(PathBuf),
    /// The pack is sourced remotely. It might be cached!
    Git(String),
}

const UNIVERSE_MANIFEST_NAME: &str = "universe.ron";

crate::impl_id!(UniverseDefinition);

impl Universe {
    pub fn from_directory(universe_dir_path: PathBuf) -> std::io::Result<Self> {
        let manifest = std::fs::read_to_string(universe_dir_path.join(UNIVERSE_MANIFEST_NAME))?;
        let manifest: UniverseDefinition =
            ron::from_str(manifest.as_str()).expect("Failed to parse manifest...");

        let mut data_packs = IndexSet::new();

        for pack in manifest.data_packs {
            match pack {
                DataPackRef::Local(data_pack_dir_path) => {
                    let data_pack =
                        DataPack::from_directory(universe_dir_path.join(data_pack_dir_path))?;
                    data_packs.insert(data_pack);
                }
                DataPackRef::Git(_) => {
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
