use std::path::PathBuf;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::data_packs::definitions::Identifiable;

/// Defines a new material!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDefinition {
    pub id: String,
    pub display_name: String,
    pub albedo: TextureRef,
}

impl Identifiable for MaterialDefinition {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRef {
    pub id: String,
}

/// Defines a new texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDefinition {
    pub id: String,
    pub display_name: String,
    pub source: PathBuf,
}

impl Identifiable for TextureDefinition {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureRef {
    pub id: String,
    pub rect: Option<Rect>,
}

/// A humble rectangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

/// Defines a new voxel!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTypeDefinition {
    pub id: String,
    pub display_name: String,
    pub appearance: BlockAppearance,
}

impl Identifiable for BlockTypeDefinition {
    fn id(&self) -> String {
        self.id.clone()
    }
}

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
