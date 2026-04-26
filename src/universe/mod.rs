//! # Universes
//!
//! Since the engine is data-driven, a Universe represents a set of
//! definitions — all the existing blocks and items, systems, etc.

use wgpu::naga::Arena;

use crate::{
    block::{BlockCluster, BlockDefinition},
    Id,
};
use std::collections::HashMap;

/// See the module-level documentation.
pub struct Universe {
    pub block_definitions: HashMap<Id, BlockDefinition>,
}

/// A World contains blocks, clusters and entities.
pub struct World {
    pub clusters: Arena<BlockCluster>,
}
