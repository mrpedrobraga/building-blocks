//! # Server
//!
//! Contains traits for the "host" of a universe and worlds where players (clients) can all play together :-)

use crate::{
    client::messages::ClientRequest,
    resources::universe::Universe,
    server::messages::{ServerConnectionMessage, ServerMessage, ServerWorldMessage},
};
use smol::channel::{Receiver, RecvError, Sender};
use std::{collections::HashMap, sync::Arc};
use tracing::{info, info_span};

pub mod messages;

/// See the module-level documentation.
pub struct UniverseServer {
    pub universe: Universe,
    pub scripting: Scripting,
    pub clients: HashMap<ClientInfo, ClientInterface>,
    pub unknown_msg_stream: Receiver<UnknownMessage>,
}

pub enum UnknownMessage {
    Connect {
        meta: ClientInfo,
        interface: ClientInterface,
    },
}

pub struct ClientInterface {
    #[allow(unused)]
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
    pub fn new(universe: Universe, unknown_msg_stream: Receiver<UnknownMessage>) -> Self {
        let scripting = Scripting::new();

        Self {
            universe,
            scripting,
            clients: HashMap::new(),
            unknown_msg_stream,
        }
    }

    pub async fn run(mut self) {
        let s = info_span!("server");
        let _ = s.enter();

        info!("[Server] Starting...");

        // Need to receive messages from the clients and print them!

        while let Ok(unknown_message) = self.unknown_msg_stream.recv().await {
            match unknown_message {
                UnknownMessage::Connect { meta, interface } => {
                    // TODO: Actually take the client info into account!
                    let _ = meta;
                    info!(
                        "[Server] Client {:?} attempting to connect. Let's see...",
                        meta.id
                    );
                    interface
                        .server_msg_tx
                        .send(ServerMessage::Connection(
                            ServerConnectionMessage::Connect {},
                        ))
                        .await
                        .expect("Failed to send connection acknowledgement to client.");
                    info!(
                        "[Server] This looks alright. Accepting {:?}'s connection request.",
                        meta.id
                    );

                    interface
                        .server_msg_tx
                        .send(ServerMessage::World(ServerWorldMessage::EnterWorld {
                            id: "default".to_string(),
                        }))
                        .await
                        .expect("Failed to send enter world message...");

                    self.clients.insert(meta, interface);
                }
            }
        }

        info!("[Server] Done.")
    }
}

/// Object that adds functionality to a server!
///
/// This beautiful baby boy is res
#[allow(unused)]
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

impl Default for Scripting {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientInfo {
    pub id: String,
}

/// A client does not hold an instance of [Server] directly.
/// Instead, a [`ServerAdapter`] abstracts over those interactions
/// so clients can connect to a server that's either hosted locally or
/// elsewhere.
#[async_trait::async_trait]
pub trait ServerInterface {
    async fn recv(&self) -> Result<ServerMessage, RecvError>;
}

/// A server that's "right here" in the same machine and process as the client.
pub struct LocalServerInterface {
    client_channels: (Sender<ClientRequest>, Receiver<ServerMessage>),
}

impl LocalServerInterface {
    pub async fn new(client: &ClientInfo, channel: &Sender<UnknownMessage>) -> Self {
        let (client_msg_tx, client_msg_rx) = smol::channel::unbounded();

        let (server_msg_tx, server_msg_rx) = smol::channel::unbounded();
        let interface = ClientInterface {
            client_msg_rx,
            server_msg_tx,
        };

        channel
            .send(UnknownMessage::Connect {
                meta: client.clone(),
                interface,
            })
            .await
            .expect("Couldn't send connection message to server...");

        Self {
            client_channels: (client_msg_tx, server_msg_rx),
        }
    }
}

#[async_trait::async_trait]
impl ServerInterface for LocalServerInterface {
    async fn recv(&self) -> Result<ServerMessage, RecvError> {
        self.client_channels.1.recv().await
    }
}

/// A server that's being hosted somewhere else.
/// Queries are resolved through a network layer like HTTP or WebSocket.
pub struct RemoteServer {}

#[async_trait::async_trait]
impl ServerInterface for RemoteServer {
    async fn recv(&self) -> Result<ServerMessage, RecvError> {
        Err(RecvError)
    }
}

pub enum RemoteConnectionError {
    Unknown,
}
