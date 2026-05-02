//! # Universes
//!
//! Since the engine is data-driven, a Universe represents all the "things"
//! that you can use to create your worlds.
//!
//! Those things are

pub mod definitions;
pub mod example;

use crate::{
    data_packs::definitions::Identifiable,
    models::{BlockTypeDefinition, MaterialDefinition, TextureDefinition},
};
use indexmap::{IndexMap, IndexSet};
use std::hash::Hash;

/// See the module-level documentation.
#[derive(Clone, Debug)]
pub struct Universe {
    pub id: String,
    pub data_packs: IndexSet<DataPack>,
}

/// A collection of resources used to build worlds with.
#[derive(Clone, Debug)]
pub struct DataPack {
    pub id: String,
    pub block_definitions: IndexMap<String, BlockTypeDefinition>,
    pub material_definitions: IndexMap<String, MaterialDefinition>,
    pub texture_definitions: IndexMap<String, TextureDefinition>,
}

impl Identifiable for DataPack {
    fn id(&self) -> String {
        self.id.clone()
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
