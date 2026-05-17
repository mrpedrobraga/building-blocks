//! # Views
//!
//! While the server holds the ground truth for a game's world, behaviour and resources,
//! the client needs to know how that world looks like in order to interact with.
//! These *View types are exactly that, the client's running "understanding" of the world.
//!
//! While connected to a server, the client will continuously update the views to
//! match the new estimated state in the server.

// TODO: Maybe instead of keeping track only of current resources,
// we should install resources into "cache" and refer to them with some indirection.
//
// This should be a good performance optimization if the player is often travelling
// between different scenes or if in a scene several layouts are reused.

use crate::world::block::BlockType;
use dashmap::DashMap;
use glam::{Quat, Vec3};

pub type LayoutId = String;
pub type BlockTypeId = String;

pub mod patch;

#[derive(Debug)]
pub struct GameView {
    pub current_universe: UniverseView,
    pub current_world: WorldView,
}

#[derive(Debug)]
pub struct UniverseView {
    pub block_types_list: DashMap<BlockTypeId, BlockType>,
}

#[derive(Debug)]
pub struct WorldView {
    pub current_scene: SceneView,
    pub layout_views: DashMap<LayoutId, LayoutView>,
}

#[derive(Debug)]
pub struct SceneView {
    pub environment: EnvironmentView,
    pub root_layout: LayoutView,
}

#[derive(Debug)]
pub struct EnvironmentView {
    pub xy_plane_is_solid_floor: bool,
}

#[derive(Debug)]
pub struct LayoutView {
    pub block_groups: Vec<BlockGroupView>,
    pub sub_layouts: Vec<LayoutRef>,
}

#[derive(Debug)]
pub struct LayoutRef {
    pub id: LayoutId,
    pub transform: glam::Mat4,
}

#[derive(Debug)]
pub struct BlockGroupView {
    pub position: Vec3,
    pub size: Vec3,
    pub orientation: Quat,
    pub block_data: Vec<BlockEntry>,
}

#[derive(Debug)]
pub struct BlockEntry {
    pub index_in_block_palette: u32,
}
