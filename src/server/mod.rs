//! # Server
//!
//! Contains traits for the "host" of a universe and worlds where players (clients) can all play together :-)

use std::sync::Arc;

use tracing::{info, info_span};

use crate::universe::Universe;

/// See the module-level documentation.
#[derive(Clone)]
pub struct UniverseServer {
    pub universe: Universe,
}

impl UniverseServer {
    pub fn new(universe: Universe) -> Self {
        Self { universe }
    }

    pub async fn run(self: Arc<Self>) {
        let s = info_span!("server");
        let _ = s.enter();

        info!("Server running!")
    }

    /// Requests this server accepts a client.
    pub fn request_client_connection(_client_metadata: ClientMetadata) -> Result<(), ()> {
        // TODO: Call the scripting language here.
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ClientMetadata {}

/// A client does not hold an instance of [Server] directly.
/// Instead, a [`ServerAdapter`] abstracts over those interactions
/// so clients can connect to a server that's either hosted locally or
/// elsewhere.
pub trait ServerAdapter {
    fn request_connection(&mut self, client: ClientMetadata) -> Result<(), ()>;

    fn get_universe(&self) -> Result<Universe, ()>;
}

/// A server that's "right here" in the same machine and process as the client.
pub struct LocalServer {
    #[allow(unused)]
    server: Arc<UniverseServer>,
}

impl LocalServer {
    pub fn new(server: Arc<UniverseServer>) -> Self {
        LocalServer { server }
    }
}

impl ServerAdapter for LocalServer {
    fn request_connection(&mut self, _client: ClientMetadata) -> Result<(), ()> {
        Ok(())
    }

    fn get_universe(&self) -> Result<Universe, ()> {
        Ok(self.server.universe.clone())
    }
}

/// A server that's being hosted somewhere else.
/// Queries are resolved through a network layer like HTTP or WebSocket.
pub struct RemoteServer {}

impl ServerAdapter for RemoteServer {
    fn request_connection(&mut self, _client: ClientMetadata) -> Result<(), ()> {
        // TODO: Talk to the server over HTTP, get a response, then upgrade the connection to WebSocket.
        Err(())
    }

    fn get_universe(&self) -> Result<Universe, ()> {
        Err(())
    }
}

pub enum RemoteConnectionError {
    Unknown,
}
