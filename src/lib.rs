//! # Building Blocks
//!
//! An unusual game engine for a very specific niche: a block-based sandbox game where
//! you can build your own worlds and play them with friends.
//!
//! There are no hard-coded things in this engine—even the blocks are data-driven!
//! Instead you install "data packs" that add block and object definitions and build with them, on top of the engine
//! that handles rendering, input, audio, networking, cross-platformness, etc, for you.

/// Anything to do with blocks or block groups!
pub mod block;

/// Things to do with data packs and the resources worldmakers can "make" their "worlds" with!
pub mod data_packs;

/// Anything to do with messaging between the "server" and the "clients".
pub mod messaging;

// TODO: Refactor where these go:

pub mod client;
pub mod models;
pub mod server;
