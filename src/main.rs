use building_blocks::{
    client::{app::Application, gui::Client},
    data_packs::Universe,
    server::{ClientInfo, UniverseServer},
};

fn main() {
    tracing_subscriber::fmt::init();

    let (app_msg_tx, app_msg_rx) = smol::channel::unbounded();

    std::thread::spawn(|| {
        let (unknown_msg_tx, unknown_msg_rx) = smol::channel::unbounded();

        let server_task = async move {
            let universe = Universe::example();
            let server = UniverseServer::new(universe, unknown_msg_rx);

            server.run().await;
        };

        let client_task = async move {
            let mut client = Client::new(
                ClientInfo {
                    id: "mrpedrobraga".to_string(),
                },
                app_msg_rx,
            );
            client.send_connection_request_local(&unknown_msg_tx).await;
            client.run().await;
        };

        smol::block_on(async {
            futures::join!(server_task, client_task);
        });
    });

    Application::new(app_msg_tx).run();
}
