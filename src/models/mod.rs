use std::path::PathBuf;

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Defines that a universe exists in a folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseDefinition {
    id: String,
    display_name: String,
}

/// Defines that a world exists in a folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDefinition {
    id: String,
    display_name: String,
}

/// Defines a new material!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDefinition {
    id: String,
    display_name: String,
    albedo: TextureRef,
}

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRef {
    id: String,
}

/// Defines a new texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDefinition {
    id: String,
    display_name: String,
    source: PathBuf,
}

/// Refers to an existing loaded texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureRef {
    id: String,
    rect: Option<Rect>,
}

/// A humble rectangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    position: Vec2,
    size: Vec2,
}

/// Defines a new voxel!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTypeDefinition {
    id: String,
    display_name: String,
    appearance: BlockAppearance,
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
