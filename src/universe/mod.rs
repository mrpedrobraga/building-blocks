//! # Universes
//!
//! Since the engine is data-driven, a Universe represents a set of
//! definitions — all the existing blocks and items, systems, etc.

use crate::{block::BlockDefinition, Id};
use std::collections::HashMap;

pub struct Universe {
    pub block_definitions: HashMap<Id, BlockDefinition>,
}
