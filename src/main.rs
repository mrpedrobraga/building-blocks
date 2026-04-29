use std::sync::Arc;

use building_blocks::{
    client::{Client, GuiClient},
    server::{LocalServer, UniverseServer},
    universe::Universe,
};

fn main() {
    tracing_subscriber::fmt::init();

    let universe = Universe::example();
    let server = UniverseServer::new(universe);
    let server = Arc::new(server);

    let server_backend = server.clone();
    std::thread::spawn(|| smol::block_on(server_backend.run()));

    let adapter = LocalServer::new(server);
    let mut client = GuiClient::new();
    client.try_connect(adapter).unwrap();
    client.run();
}
