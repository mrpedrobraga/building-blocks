//! # Client
//!
//! Traits and structs for clients which are a player's way of interacting with a universe.

use crate::{
    client::gui::Application,
    server::{ClientMetadata, ServerAdapter},
    universe::{Scene, Universe, World},
};

pub mod gui;

pub trait Client {
    fn try_connect<S: ServerAdapter + 'static>(&mut self, server: S) -> Result<(), ()>;

    fn metadata(&self) -> ClientMetadata;
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
    fn try_connect<S: ServerAdapter>(&mut self, #[expect(unused)] server: S) -> Result<(), ()> {
        Ok(())
    }

    fn metadata(&self) -> ClientMetadata {
        ClientMetadata {}
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

    /// Runs the application.
    ///
    /// Technically, all you need to run such a client is this:
    ///
    /// ```rust
    /// DefaultClient::new().run()
    /// ```
    ///
    /// Though you can call functions on the client before calling run, for example, you might
    /// preemptively connect to some server.
    ///
    /// ## Multi-threading
    ///
    /// Pretty please call this on the main thread (winit needs this).
    pub fn run(&mut self) {
        Application::run(self);
    }
}

impl Client for GuiClient {
    fn try_connect<S: ServerAdapter + 'static>(
        &mut self,
        #[allow(unused)] server_adapter: S,
    ) -> Result<(), ()> {
        self.server_adapter.replace(Box::new(server_adapter));
        Ok(())
    }

    fn metadata(&self) -> ClientMetadata {
        ClientMetadata {}
    }
}
