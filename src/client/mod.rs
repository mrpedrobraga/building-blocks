//! # Client
//!
//! Traits and structs for clients which are players' way of interacting with a universe through a central authority (a server).

use self::render::gpu::GameRenderer;
use crate::{
    client::{app::AppMessage, render::GameRenderState, view::GameView},
    server::{
        ClientInfo, LocalServerInterface, ServerInterface, UnknownMessage, messages::ServerMessage,
    },
};
use glam::UVec2;
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
    pub game_render_state: Option<GameRenderState>,
    pub game_renderer: Option<GameRenderer>,
}

impl Client {
    pub fn new(info: ClientInfo, app_msg_rx: Receiver<AppMessage>) -> Self {
        Self {
            info,
            server_interface: None,
            game_view: GameView::new(),
            game_render_state: None,
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
            let (Some(server_interface), Some(app_msg_rx)) =
                (self.server_interface.as_ref(), self.app_msg_rx.as_mut())
            else {
                break;
            };

            enum Message {
                App(AppMessage),
                Server(ServerMessage),
            }
            let msg = smol::future::race(
                async { server_interface.next_message().await.map(Message::Server) },
                async { app_msg_rx.recv().await.map(Message::App) },
            )
            .await;

            let mut should_redraw = false;

            match msg {
                Ok(Message::App(app_message)) => Self::handle_app_message(
                    &mut self.game_render_state,
                    &mut self.game_renderer,
                    app_message,
                    &mut should_redraw,
                ),
                Ok(Message::Server(server_message)) => {
                    self.handle_server_message(server_message).await
                }
                Err(err) => {
                    info!("[Client] Shutting down. Cause: {:?}", err);
                    break;
                }
            };

            /* Eagerly draining all accumulated messages. */
            if let Some(app_msg_rx) = self.app_msg_rx.as_ref() {
                while let Ok(extra_app_message) = app_msg_rx.try_recv() {
                    Self::handle_app_message(
                        &mut self.game_render_state,
                        &mut self.game_renderer,
                        extra_app_message,
                        &mut should_redraw,
                    );
                }
            }

            /* Only rendering at the very end... in the future, maybe we should render on another thread. */
            if should_redraw
                && let Some(game_renderer) = self.game_renderer.as_mut()
                && let Some(game_render_state) = self.game_render_state.as_mut()
            {
                let screen_size = game_renderer.render_target.surface_size;
                game_render_state.world_state.tick(
                    &game_renderer.gpu,
                    UVec2::new(screen_size.width, screen_size.height),
                );
                game_renderer.render(game_render_state);
            }
        }

        info!("[Client] Done.");
    }

    /// Handles a message send by an [`Application`]...
    pub fn handle_app_message(
        game_render_state: &mut Option<GameRenderState>,
        game_renderer: &mut Option<GameRenderer>,
        app_message: AppMessage,
        should_redraw: &mut bool,
    ) {
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
                // TODO: Pass in the current GameView so we start with some data,
                // then after that we do progressive patching.
                *game_render_state = Some(GameRenderState::new(&gpu));
                *game_renderer = Some(GameRenderer::new(gpu, window_render_target));
            }
            AppMessage::ResizeRenderTarget(new_size) => {
                if let Some(game_renderer) = game_renderer.as_mut() {
                    game_renderer.resize(new_size);
                }
            }
            AppMessage::PleaseRender => {
                *should_redraw = true;
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
