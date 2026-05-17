use std::{fs, path::PathBuf};

use axum::{Router, extract::State, response::{Html, IntoResponse}, routing::get};
use fastwebsockets::upgrade::UpgradeFut;
use self::info::ServerMetadata;

pub mod info;

const IP: &str = "0.0.0.0";
const PORT: &str = "3000";

#[tokio::main]
async fn main() {
    let path = PathBuf::from("./examples/example_server").join("server.ron");
    let raw = fs::read_to_string(path).unwrap();
    let meta: ServerMetadata = ron::from_str(raw.as_str()).unwrap();

    boot_up(&meta).await;

    let app = Router::new()
        .route("/", get(home_handler))
        .route("/ws", get(ws_connection_request))
        .with_state(meta);

    let listener = tokio::net::TcpListener::bind(
        format!("{IP}:{PORT}")
    ).await.unwrap();

    println!("[Server] Listening at {IP}:{PORT}.");
    axum::serve(listener, app).await.unwrap();
}

async fn boot_up(meta: &ServerMetadata) {
    println!("[Server] Initializing...");
    println!("[Server] Server '{}' reporting for duty.", meta.name)
}

async fn home_handler(State(server_meta): State<ServerMetadata>) -> impl IntoResponse {
    let html = format!("<h1>Welcome to {}!</h1>", server_meta.name);
    
    Html(html)
}

async fn ws_connection_request(ws: fastwebsockets::upgrade::IncomingUpgrade) -> impl IntoResponse {
    let (response, fut) = ws.upgrade().expect("Failed to upgrade Websocket connection.");

    tokio::task::spawn(async move {
        if let Err(e) = handle_client(fut).await {
            eprintln!("[Server] Websocket connection error: {e}.")
        }
    });

    response
}

async fn handle_client(fut: UpgradeFut) -> Result<(), fastwebsockets::WebSocketError> {
    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);

    loop {
        let frame = ws.read_frame().await?;
        match frame.opcode {
            fastwebsockets::OpCode::Text => {
                // Reply with the same exact message >:-)
                ws.write_frame(frame).await?;
            },
            fastwebsockets::OpCode::Close => {
                println!("[Server] Closing connection.");
                break;
            },
            _ => unimplemented!()
        }
    }

    Ok(())
}