#![allow(unused)]

use std::sync::{mpsc, Arc};

use building_blocks::{
    client::{gui::Application, Client, GuiClient},
    data_packs::Universe,
    server::{ClientInfo, LocalServerInterface, ServerAdapter, UniverseServer},
};

fn main() {
    tracing_subscriber::fmt::init();

    let (client_msg_tx, client_msg_rx) = mpsc::channel();
    let (server_msg_tx, server_msg_rx) = mpsc::channel();

    let (app_msg_tx, app_msg_rx) = mpsc::channel();

    std::thread::spawn(|| {
        let universe = Universe::example();
        let server = UniverseServer::new(universe);
        let server = Arc::new(server);
        smol::block_on(async {
            server.run().await;
        });
    });

    let mut adapter = LocalServerInterface::new((client_msg_tx, server_msg_rx));
    let mut client = GuiClient::new(app_msg_rx);
    adapter
        .request_connection(ClientInfo {})
        .expect("Failed sending connection request.\nUnderstand — the server didn't 'reject' the client per se, the request failed to reach it altogether.");
    client.install_adapter(adapter).unwrap();
    Application::new(app_msg_tx).run();
}
