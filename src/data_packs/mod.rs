//! # Universes
//!
//! Since the engine is data-driven, a Universe represents a set of
//! definitions — all the existing blocks and items, systems, etc.

pub mod example;

use indexmap::IndexMap;

use crate::world::block::BlockDefinition;

/// See the module-level documentation.
#[derive(Clone)]
pub struct Universe {
    pub block_definitions: IndexMap<String, BlockDefinition>,
}
