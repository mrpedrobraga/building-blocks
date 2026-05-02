use std::sync::mpsc;

use building_blocks::{
    client::{gui::Application, Client, GuiClient},
    data_packs::Universe,
    server::{ClientInfo, ClientInterface, LocalServerInterface, ServerAdapter, UniverseServer},
};

fn main() {
    tracing_subscriber::fmt::init();

    let (client_msg_tx, client_msg_rx) = mpsc::channel();
    let (server_msg_tx, server_msg_rx) = mpsc::channel();

    let (app_msg_tx, app_msg_rx) = mpsc::channel();

    std::thread::spawn(|| {
        let universe = Universe::example();
        let mut server = UniverseServer::new(universe);

        let client_interface = ClientInterface::new(client_msg_rx, server_msg_tx);

        server
            .request_client_connection(ClientInfo {}, client_interface)
            .unwrap();

        smol::block_on(async {
            server.run().await;
        });
    });

    std::thread::spawn(move || {
        let mut adapter = LocalServerInterface::new((client_msg_tx, server_msg_rx));
        let mut client = GuiClient::new(app_msg_rx);
        adapter
            .request_connection(ClientInfo {})
            .expect("Failed sending connection request.\nUnderstand — the server didn't 'reject' the client per se, the request failed to reach it altogether.");
        client.install_adapter(adapter).unwrap();
        client.run();
    });

    Application::new(app_msg_tx).run();
}
