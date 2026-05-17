use crate::{impl_id, world::block::BlockAppearance};
use serde::{Deserialize, Serialize};

/// Defines a new voxel!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTypeDefinition {
    pub id: String,
    pub display_name: String,
    pub appearance: BlockAppearance,
}

impl_id!(BlockTypeDefinition);

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTypeRef {
    id: String,
}
