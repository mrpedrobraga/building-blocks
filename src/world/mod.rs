//! # Worlds
//!
//! A world is the stage of one's creativity — it uses resources declared in a [`Universe`]
//! and arranges them together into something beautiful.

/// All to do with the blocks you use to create builds and landscapes, yeah!
pub mod block;
pub mod example;

use block::BlockGroup;
use generational_arena::Arena;
use indexmap::IndexMap;

/// A World is a collection of scenes plus some metadata.
pub struct World {
    pub id: String,
    pub scenes: IndexMap<String, Scene>,
}

/// A scene is basically [`Layout`] with an added environment.
#[derive(Clone)]
pub struct Scene {
    pub environment: Environment,
    pub root_layout: Layout,
}

/// The environment of a scene.
///
/// Futurely, this will contain per scene metadata such as
/// the skybox, lighting, which shaders to use, etc.
#[derive(Clone)]
pub struct Environment {
    /// Whether the xy plane (z = 0) should act as a solid, impassable floor in regards to physics objects.
    pub xy_plane_is_solid_floor: bool,
}

/// A layout is a reusable collection of block groups, entities, etc.
///
/// Layouts will be tipically nested and arranged together to create bigger layouts.
/// Adding an [`Environment`] to a Layout can get you a scene, a
#[derive(Clone)]
pub struct Layout {
    pub block_groups: Arena<BlockGroup>,
}
