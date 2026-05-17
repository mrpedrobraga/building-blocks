use crate::impl_id;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Defines a new texture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDefinition {
    pub id: String,
    pub display_name: String,
    pub source: PathBuf,
}

impl_id!(TextureDefinition);

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
