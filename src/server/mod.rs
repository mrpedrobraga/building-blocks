#![allow(unused)]
//! # Server
//!
//! Contains traits for the "host" of a universe and worlds where players (clients) can all play together :-)

use crate::{
    data_packs::Universe,
    messaging::server::{ServerConnectionMessage, ServerMessage},
};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};
use tracing::info_span;

/// See the module-level documentation.
pub struct UniverseServer {
    pub universe: Universe,
    pub scripting: Scripting,
    pub clients: HashMap<ClientInfo, ClientInterface>,
}

pub struct ClientInterface {
    client_msg_rx: Receiver<ClientRequest>,
    server_msg_tx: Sender<ServerMessage>,
}

impl ClientInterface {
    pub fn new(
        client_msg_rx: Receiver<ClientRequest>,
        server_msg_tx: Sender<ServerMessage>,
    ) -> Self {
        Self {
            client_msg_rx,
            server_msg_tx,
        }
    }
}

impl UniverseServer {
    pub fn new(universe: Universe) -> Self {
        let scripting = Scripting::new();

        Self {
            universe,
            scripting,
            clients: HashMap::new(),
        }
    }

    pub async fn run(self) {
        let s = info_span!("server");
        let _ = s.enter();
    }

    /// Requests this server accepts a client.
    pub fn request_client_connection(
        &mut self,
        client_metadata: ClientInfo,
        client_interface: ClientInterface,
    ) -> Result<(), ()> {
        // let mut vm = rune::Vm::new(self.scripting.runtime.clone(), self.scripting.unit.clone());

        // vm.call(["Server", "client_requested_connection"], ((), ()))
        //     .unwrap();

        client_interface
            .server_msg_tx
            .send(ServerMessage::Connection(
                ServerConnectionMessage::Connect {},
            ))
            .expect("Failed to send connection message to client!");

        self.clients.insert(client_metadata, client_interface);

        Ok(())
    }
}

/// Object that adds functionality to a server!
///
/// This beautiful baby boy is res
pub struct Scripting {
    unit: Arc<rune::Unit>,
    runtime: Arc<rune::runtime::RuntimeContext>,
}

impl Scripting {
    pub fn new() -> Self {
        let mut server_module = rune::Module::new();
        server_module
            .constant("TURN", std::f32::consts::TAU)
            .build()
            .unwrap()
            .docs(rune::docstring! {
               /// Represents a full turn in radians
            })
            .unwrap();

        let mut context = rune::Context::with_default_modules().unwrap();
        context.install(server_module).unwrap();

        let runtime = context.runtime().unwrap();
        let runtime = Arc::new(runtime);

        let mut sources = rune::Sources::new();
        sources
            .insert(rune::Source::from_path("./src/server/test.rn").unwrap())
            .unwrap();

        let mut diagnostics = rune::Diagnostics::new();

        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            let mut writer =
                rune::termcolor::StandardStream::stderr(rune::termcolor::ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources).unwrap();
        }

        let unit = result.unwrap();
        let unit = Arc::new(unit);

        Self { unit, runtime }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientInfo {}

/// A client does not hold an instance of [Server] directly.
/// Instead, a [`ServerAdapter`] abstracts over those interactions
/// so clients can connect to a server that's either hosted locally or
/// elsewhere.
pub trait ServerAdapter {
    fn request_connection(&mut self, client: ClientInfo) -> Result<(), ()>;

    fn next_message(&self) -> Result<ServerMessage, ()>;
}

#[derive(Debug)]
pub enum ClientRequest {
    Connect,
}

/// A server that's "right here" in the same machine and process as the client.
pub struct LocalServerInterface {
    client_channels: (Sender<ClientRequest>, Receiver<ServerMessage>),
}

impl LocalServerInterface {
    pub fn new(client_channels: (Sender<ClientRequest>, Receiver<ServerMessage>)) -> Self {
        LocalServerInterface { client_channels }
    }
}

impl ServerAdapter for LocalServerInterface {
    fn request_connection(&mut self, _client: ClientInfo) -> Result<(), ()> {
        self.client_channels
            .0
            .send(ClientRequest::Connect)
            .expect("Couldn't send request.");
        Ok(())
    }

    fn next_message(&self) -> Result<ServerMessage, ()> {
        self.client_channels.1.recv().map_err(|_| ())
    }
}

/// A server that's being hosted somewhere else.
/// Queries are resolved through a network layer like HTTP or WebSocket.
pub struct RemoteServer {}

impl ServerAdapter for RemoteServer {
    fn request_connection(&mut self, _client: ClientInfo) -> Result<(), ()> {
        // TODO: Talk to the server over HTTP, get a response, then upgrade the connection to WebSocket.
        Err(())
    }

    fn next_message(&self) -> Result<ServerMessage, ()> {
        Err(())
    }
}

pub enum RemoteConnectionError {
    Unknown,
}
