//! # Server
//!
//! Contains traits for the "host" of a universe and worlds where players (clients) can all play together :-)

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
}

/// A server that's "right here" in the same machine and process as the client.
pub struct LocalServer {
    #[allow(unused)]
    server: UniverseServer,
}

impl LocalServer {
    pub fn new(server: UniverseServer) -> Self {
        LocalServer { server }
    }
}

impl ServerAdapter for LocalServer {
    fn request_connection(&mut self, _client: ClientMetadata) -> Result<(), ()> {
        Ok(())
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
}

pub enum RemoteConnectionError {
    Unknown,
}
