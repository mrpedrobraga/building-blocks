use building_blocks::application::Application;

fn main() {
    tracing_subscriber::fmt::init();

    Application::run();
}
