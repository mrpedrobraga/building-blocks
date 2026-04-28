//! # Client
//!
//! Traits and structs for clients which are a player's way of interacting with a universe.

use crate::{client::application::Application, server::ServerInterface};

pub mod application;

pub trait Client {
    fn try_connect<S: ServerInterface>(&mut self, server: S) -> Result<(), ()>;
}

pub struct DummyClient {}

impl DummyClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Client for DummyClient {
    fn try_connect<S: ServerInterface>(&mut self, #[expect(unused)] server: S) -> Result<(), ()> {
        Ok(())
    }
}

/// The "default" GUI client.
///
/// It launches a desktop application through which you can connect to servers and stuff.
pub struct GuiClient {}

impl GuiClient {
    pub fn new() -> Self {
        Self {}
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
    fn try_connect<S: ServerInterface>(&mut self, #[allow(unused)] server: S) -> Result<(), ()> {
        Ok(())
    }
}
