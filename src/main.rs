use building_blocks::client::GuiClient;

fn main() {
    tracing_subscriber::fmt::init();

    let mut client = GuiClient::new();
    client.run();
}
