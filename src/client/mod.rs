//! # Client
//!
//! Traits and structs for clients which are a player's way of interacting with a universe.

use crate::{
    server::{ClientInfo, ServerAdapter},
    universe::{Scene, Universe, World},
};

pub mod gui;

pub trait Client {
    fn install_adapter<S: ServerAdapter + 'static>(&mut self, server: S) -> Result<(), ()>;

    fn metadata(&self) -> ClientInfo;
}

pub struct GameView {
    pub current_universe: Universe,
    pub current_world: World,
    pub current_scene: Scene,
}

pub struct DummyClient {}

impl DummyClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Client for DummyClient {
    fn install_adapter<S: ServerAdapter>(&mut self, #[expect(unused)] server: S) -> Result<(), ()> {
        Ok(())
    }

    fn metadata(&self) -> ClientInfo {
        ClientInfo {}
    }
}

/// The "default" GUI client.
///
/// It launches a desktop application through which you can connect to servers and stuff.
pub struct GuiClient {
    server_adapter: Option<Box<dyn ServerAdapter>>,
    pub game_resources: Option<GameView>,
}

impl GuiClient {
    pub fn new() -> Self {
        Self {
            server_adapter: None,
            game_resources: None,
        }
    }
}

impl Client for GuiClient {
    fn install_adapter<S: ServerAdapter + 'static>(
        &mut self,
        #[allow(unused)] server_adapter: S,
    ) -> Result<(), ()> {
        self.server_adapter.replace(Box::new(server_adapter));
        Ok(())
    }

    fn metadata(&self) -> ClientInfo {
        ClientInfo {}
    }
}
