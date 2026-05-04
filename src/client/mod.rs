//! # Client
//!
//! Traits and structs for clients which are a player's way of interacting with a universe.

use std::pin::Pin;

use smol::channel::{Receiver, Sender};
use tracing::{info, info_span};

use crate::{
    data_packs::Universe,
    server::{ClientInfo, LocalServerInterface, ServerInterface, UnknownMessage},
    world::{Scene, World},
};

pub mod gui;

pub trait Client {
    fn send_connection_request_local<'a>(
        &'a mut self,
        channel: &'a Sender<UnknownMessage>,
    ) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn info(&self) -> &ClientInfo;
}

pub struct GameView {
    pub current_universe: Universe,
    pub current_world: World,
    pub current_scene: Scene,
}

/// The "default" GUI client.
///
/// It launches a desktop application through which you can connect to servers and stuff.
pub struct GuiClient {
    pub info: ClientInfo,
    server_interface: Option<Box<dyn ServerInterface>>,
    pub game_resources: Option<GameView>,
    pub app_msg_rx: Option<Receiver<GuiMessage>>,
}

pub enum GuiMessage {
    // TODO: Create a new bespoke event type? Maybe? Perhaps steal the type from `ui-composer`?
    WindowEvent(winit::event::WindowEvent),
    DeviceEvent(winit::event::DeviceEvent),
    Redraw,
}

impl GuiClient {
    pub fn new(info: ClientInfo, app_msg_rx: Receiver<GuiMessage>) -> Self {
        Self {
            info,
            server_interface: None,
            game_resources: None,
            app_msg_rx: Some(app_msg_rx),
        }
    }

    pub async fn run(&mut self) {
        let s = info_span!("client");
        let _ = s.enter();

        let Some(interface) = self.server_interface.as_ref() else {
            return;
        };

        info!("[Client] Waiting for server messages.");

        loop {
            match interface.recv().await {
                Ok(message) => {
                    info!("[Client] Message from server --- {:?}", message);
                }
                Err(err) => {
                    info!("[Client] Shutting down. Cause: {:?}", err);
                    break;
                }
            }
        }

        info!("[Client] Done.");
    }
}

impl Client for GuiClient {
    fn send_connection_request_local<'a>(
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

    fn info(&self) -> &ClientInfo {
        &self.info
    }
}
