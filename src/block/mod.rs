use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BlockDefinition {
    display_name: String,
    color: Vec3,
}
