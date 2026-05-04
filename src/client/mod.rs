//! # Client
//!
//! Traits and structs for clients which are players' way of interacting with a universe through a central authority (a server).

use crate::{
    resources::universe::Universe,
    world::{Scene, World},
};

pub mod app;
pub mod gui;

pub struct GameView {
    pub current_universe: Universe,
    pub current_world: World,
    pub current_scene: Scene,
}
