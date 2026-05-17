#![forbid(unsafe_code)]

//! # Building Blocks
//!
//! An unusual game engine for a very specific niche: a block-based sandbox game where
//! you can build your own worlds and play them with friends.
//!
//! There are no hard-coded things in this engine—even the blocks are data-driven!
//! Instead you install "data packs" that add block and object definitions and build with them, on top of the engine
//! that handles rendering, input, audio, networking, cross-platformness, etc, for you.

/// Anything to do with the beautiful worlds you can create!
pub mod world;

/// Things to do with data packs and the resources worldmakers can "make" their "worlds" with!
pub mod resources;

/// The client, which is responsible for 'visualizing' a game and interfacing with the player's device.
pub mod client;
/// The server, the central authority of the game, which broadcasts the state of the world and evolves it.
pub mod server;

/* Helpful macros */

#[macro_export]
macro_rules! impl_id {
    ($type:ty) => {
        impl $crate::resources::Id for $type {
            fn id(&self) -> String {
                self.id.clone()
            }
        }
    };
}
