//! # Server
//!
//! Contains traits for the "host" of a universe and worlds where players (clients) can all play together :-)

use std::convert::Infallible;

#[derive(Debug, Clone)]
pub struct UniverseServer {}

impl UniverseServer {
    pub fn new() -> Self {
        Self {}
    }
}

/// A client does not hold an instance of [Server] directly.
/// Instead, a [ServerInterface] abstracts over those interactions
/// so clients can connect to a server that's either hosted locally or
/// elsewhere.
pub trait ServerInterface {
    type ConnectionError;
}

/// A server that's "right here" in the same machine and process as the client.
pub struct LocalServerInterface {
    #[allow(unused)]
    server: UniverseServer,
}

impl LocalServerInterface {
    pub fn new(server: UniverseServer) -> Self {
        LocalServerInterface { server }
    }
}

impl ServerInterface for LocalServerInterface {
    type ConnectionError = Infallible;
}

/// A server that's being hosted somewhere else.
/// Queries are resolved through a network layer like HTTP or WebSocket.
pub struct RemoteServerInterface {}

impl ServerInterface for RemoteServerInterface {
    type ConnectionError = RemoteConnectionError;
}

pub enum RemoteConnectionError {
    Unknown,
}
