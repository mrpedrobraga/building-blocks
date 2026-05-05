//! # Client
//!
//! Traits and structs for clients which are players' way of interacting with a universe through a central authority (a server).

use self::render::gpu::GameRenderResources;
use crate::{
    client::{app::AppMessage, render::GameRenderState, view::GameView},
    server::{
        ClientInfo, LocalServerInterface, ServerInterface, UnknownMessage, messages::ServerMessage,
    },
};
use smol::channel::{Receiver, Sender};
use std::pin::Pin;
use tracing::{info, info_span, trace};

pub mod app;
pub mod messages;
pub mod render;
pub mod view;

/// The "default" GUI client.
///
/// It launches a desktop application through which you can connect to servers and stuff.
pub struct Client {
    pub info: ClientInfo,
    pub server_interface: Option<Box<dyn ServerInterface>>,
    pub app_msg_rx: Option<Receiver<AppMessage>>,
    pub game_view: GameView,
    pub game_render_state: GameRenderState,
    pub game_renderer: Option<GameRenderResources>,
}

impl Client {
    pub fn new(info: ClientInfo, app_msg_rx: Receiver<AppMessage>) -> Self {
        Self {
            info,
            server_interface: None,
            game_view: GameView::new(),
            game_render_state: GameRenderState::new(),
            game_renderer: None,
            app_msg_rx: Some(app_msg_rx),
        }
    }

    pub fn info(&self) -> &ClientInfo {
        &self.info
    }

    pub fn send_connection_request_local<'a>(
        &'a mut self,
        channel: &'a Sender<UnknownMessage>,
    ) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        info!("[Client] Attempting to establish a connection to the server...");
        Box::pin(async {
            self.server_interface.replace(Box::new(
                LocalServerInterface::new(self.info(), channel).await,
            ));
        })
    }

    pub async fn run(&mut self) {
        let s = info_span!("client");
        let _ = s.enter();

        info!("[Client] Waiting for server messages.");

        loop {
            let (Some(interface), Some(app_msg_rx)) =
                (self.server_interface.as_ref(), self.app_msg_rx.as_ref())
            else {
                break;
            };

            enum Message {
                App(AppMessage),
                Server(ServerMessage),
            }
            let msg = smol::future::race(
                async { interface.recv().await.map(Message::Server) },
                async { app_msg_rx.recv().await.map(Message::App) },
            )
            .await;

            match msg {
                Ok(Message::App(app_message)) => self.handle_app_message(app_message).await,
                Ok(Message::Server(server_message)) => {
                    self.handle_server_message(server_message).await
                }
                Err(err) => {
                    info!("[Client] Shutting down. Cause: {:?}", err);
                    break;
                }
            };
        }

        info!("[Client] Done.");
    }

    /// Handles a message send by an [`Application`]...
    pub async fn handle_app_message(&mut self, app_message: AppMessage) {
        trace!("[Client] Message sent by the UI: {:?}.", app_message);

        match app_message {
            AppMessage::WindowEvent(_) => {
                // TODO: Implement!
            }
            AppMessage::DeviceEvent(_) => {
                // TODO: Implement!
                // Here we'll listen to input, take it through
                // the input map and send it to the server.
            }
            AppMessage::SetRenderTarget(gpu, window_render_target) => {
                self.game_renderer = Some(GameRenderResources::new(gpu, window_render_target));
            }
            AppMessage::ResizeRenderTarget(new_size) => {
                if let Some(game_renderer) = self.game_renderer.as_mut() {
                    game_renderer.resize(new_size);
                }
            }
            AppMessage::PleaseRender => {
                if let Some(game_renderer) = self.game_renderer.as_mut() {
                    game_renderer.render(&self.game_render_state);
                }
            }
        }
    }

    /// Handles a message send by the server...
    ///
    /// TODO: Take `&self` instead of `&mut self` and use interior mutability with async capable data structures.
    pub async fn handle_server_message(&mut self, server_message: ServerMessage) {
        use crate::server::messages::*;
        use ServerMessage::*;

        match server_message {
            Connection(server_connection_message) => match server_connection_message {
                ServerConnectionMessage::Connect {} => {
                    info!("[Client] Server accepted connection! Yay!");
                    self.game_view.reset();
                }
                ServerConnectionMessage::Disconnect { reason } => {
                    info!("[Client] Server disconnected because '{}'.", reason)
                }
            },
            World(server_world_message) => {
                info!(
                    "[Client] Received 'world' type message: {:?}",
                    server_world_message
                );

                self.game_view.current_world.patch(&server_world_message);
            }
        }
    }
}
