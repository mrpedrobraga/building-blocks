use std::sync::{mpsc, Arc};

use building_blocks::{
    client::{gui::Application, Client, GuiClient},
    server::{ClientInfo, LocalServer, ServerAdapter, UniverseServer},
    universe::Universe,
};

fn main() {
    tracing_subscriber::fmt::init();

    let (client_msg_tx, client_msg_rx) = mpsc::channel();
    let (server_msg_tx, server_msg_rx) = mpsc::channel();

    std::thread::spawn(|| {
        let universe = Universe::example();
        let server = UniverseServer::new(universe);
        let server = Arc::new(server);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            server.run().await;
        });
    });

    let mut adapter = LocalServer::new((client_msg_tx, server_msg_rx));
    let mut client = GuiClient::new();
    adapter
        .request_connection(ClientInfo {})
        .expect("Failed sending connection request.\nUnderstand — the server didn't 'reject' the client per se, the request failed to reach it altogether.");
    client.install_adapter(adapter).unwrap();
    Application::run(&mut client);
}
