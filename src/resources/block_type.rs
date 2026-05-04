use crate::{impl_id, resources::material::MaterialRef};
use serde::{Deserialize, Serialize};

/// Defines a new voxel!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTypeDefinition {
    pub id: String,
    pub display_name: String,
    pub appearance: BlockAppearance,
}

impl_id!(BlockTypeDefinition);

/// Defines the appearance of a voxel...
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockAppearance {
    Cuboid {
        x_min: MaterialRef,
        x_max: MaterialRef,
        y_min: MaterialRef,
        y_max: MaterialRef,
        z_min: MaterialRef,
        z_max: MaterialRef,
    },
}

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTypeRef {
    id: String,
}
