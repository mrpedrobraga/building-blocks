//! # Server
//!
//! Contains traits for the "host" of a universe and worlds where players (clients) can all play together :-)

use crate::universe::Universe;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};
use tracing::{info, info_span};

/// See the module-level documentation.
pub struct UniverseServer {
    pub universe: Universe,
    pub scripting: Scripting,
}

impl UniverseServer {
    pub fn new(universe: Universe) -> Self {
        let scripting = Scripting::new();

        Self {
            universe,
            scripting,
        }
    }

    pub async fn run(self: Arc<Self>) {
        let s = info_span!("server");
        let _ = s.enter();

        info!("Server running!")
    }

    /// Requests this server accepts a client.
    pub fn request_client_connection(&mut self, _client_metadata: ClientInfo) -> Result<(), ()> {
        let mut vm = rune::Vm::new(self.scripting.runtime.clone(), self.scripting.unit.clone());

        vm.call(["Server", "client_requested_connection"], ((), ()))
            .unwrap();

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

#[derive(Debug, Clone)]
pub struct ClientInfo {}

/// A client does not hold an instance of [Server] directly.
/// Instead, a [`ServerAdapter`] abstracts over those interactions
/// so clients can connect to a server that's either hosted locally or
/// elsewhere.
pub trait ServerAdapter {
    fn request_connection(&mut self, client: ClientInfo) -> Result<(), ()>;

    fn get_universe(&self) -> Result<Universe, ()>;
}

#[derive(Debug)]
pub enum ClientRequest {
    Connect,
}

#[derive(Debug)]
pub enum ServerResponse {
    Connection { status: bool },
}

/// A server that's "right here" in the same machine and process as the client.
pub struct LocalServer {
    client_channels: (Sender<ClientRequest>, Receiver<ServerResponse>),
}

impl LocalServer {
    pub fn new(client_channels: (Sender<ClientRequest>, Receiver<ServerResponse>)) -> Self {
        LocalServer { client_channels }
    }
}

impl ServerAdapter for LocalServer {
    fn request_connection(&mut self, _client: ClientInfo) -> Result<(), ()> {
        self.client_channels
            .0
            .send(ClientRequest::Connect)
            .expect("Couldn't send request.");
        Ok(())
    }

    fn get_universe(&self) -> Result<Universe, ()> {
        Err(())
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

    fn get_universe(&self) -> Result<Universe, ()> {
        Err(())
    }
}

pub enum RemoteConnectionError {
    Unknown,
}
