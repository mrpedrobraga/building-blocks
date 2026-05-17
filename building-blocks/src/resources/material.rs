use crate::{impl_id, resources::texture::TextureRef};
use serde::{Deserialize, Serialize};

/// Defines a new material!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDefinition {
    pub id: String,
    pub display_name: String,
    pub albedo: TextureRef,
}

impl_id!(MaterialDefinition);

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRef {
    pub id: String,
}
