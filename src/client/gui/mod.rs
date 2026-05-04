use crate::{
    client::GameView,
    server::{ClientInfo, LocalServerInterface, ServerInterface, UnknownMessage},
};
use smol::channel::{Receiver, Sender};
use std::pin::Pin;
use tracing::{info, info_span};

/// The "default" GUI client.
///
/// It launches a desktop application through which you can connect to servers and stuff.
pub struct Client {
    pub info: ClientInfo,
    pub server_interface: Option<Box<dyn ServerInterface>>,
    pub game_resources: Option<GameView>,
    pub app_msg_rx: Option<Receiver<AppMessage>>,
}

pub enum AppMessage {
    // TODO: Create a new bespoke event type? Maybe? Perhaps steal the type from `ui-composer`?
    WindowEvent(winit::event::WindowEvent),
    DeviceEvent(winit::event::DeviceEvent),
    Redraw,
}

impl Client {
    pub fn new(info: ClientInfo, app_msg_rx: Receiver<AppMessage>) -> Self {
        Self {
            info,
            server_interface: None,
            game_resources: None,
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

        let Some(interface) = self.server_interface.as_ref() else {
            return;
        };

        info!("[Client] Waiting for server messages.");

        loop {
            use crate::messaging::server::ServerMessage::*;

            match interface.recv().await {
                Ok(message) => match message {
                    Connection(server_connection_message) => match server_connection_message {
                        crate::messaging::server::ServerConnectionMessage::Connect {} => {
                            info!("[Client] Server accepted connection! Yay!")
                        }
                        crate::messaging::server::ServerConnectionMessage::Disconnect {
                            reason,
                        } => {
                            info!("[Client] Server disconnected because '{}'.", reason)
                        }
                    },
                    World(server_world_message) => {
                        info!(
                            "[Client] Received 'world' type message: {:?}",
                            server_world_message
                        )
                    }
                    Scene(server_scene_message) => {
                        info!(
                            "[Client] Received 'scene' type message: {:?}",
                            server_scene_message
                        )
                    }
                },
                Err(err) => {
                    info!("[Client] Shutting down. Cause: {:?}", err);
                    break;
                }
            }
        }

        info!("[Client] Done.");
    }
}
