use building_blocks::{
    client::{Client, GuiClient},
    server::{LocalServerInterface, UniverseServer},
};

fn main() {
    tracing_subscriber::fmt::init();

    let server = UniverseServer::new();
    let server_interface = LocalServerInterface::new(server.clone());

    let mut client = GuiClient::new();
    client
        .try_connect(server_interface)
        .expect("Failed to connect to local server.");
    client.run();
}
