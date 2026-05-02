//! # Client
//!
//! Traits and structs for clients which are a player's way of interacting with a universe.

use std::sync::mpsc::Receiver;

use tracing::{info, info_span};

use crate::{
    data_packs::Universe,
    server::{ClientInfo, ServerAdapter},
    world::{Scene, World},
};

pub mod gui;

pub trait Client {
    fn install_adapter<S: ServerAdapter + 'static>(&mut self, server: S) -> Result<(), ()>;

    fn metadata(&self) -> ClientInfo;
}

pub struct GameView {
    pub current_universe: Universe,
    pub current_world: World,
    pub current_scene: Scene,
}

pub struct DummyClient {}

impl DummyClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Client for DummyClient {
    fn install_adapter<S: ServerAdapter>(&mut self, #[expect(unused)] server: S) -> Result<(), ()> {
        Ok(())
    }

    fn metadata(&self) -> ClientInfo {
        ClientInfo {}
    }
}

/// The "default" GUI client.
///
/// It launches a desktop application through which you can connect to servers and stuff.
pub struct GuiClient {
    server_adapter: Option<Box<dyn ServerAdapter>>,
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
    pub fn new(app_msg_rx: Receiver<GuiMessage>) -> Self {
        Self {
            server_adapter: None,
            game_resources: None,
            app_msg_rx: Some(app_msg_rx),
        }
    }

    pub fn run(&mut self) {
        let s = info_span!("client");
        let _ = s.enter();

        if let Ok(message) = self.server_adapter.as_ref().unwrap().next_message() {
            info!("[Client] Message from server --- {:?}", message);
        }
    }
}

impl Client for GuiClient {
    fn install_adapter<S: ServerAdapter + 'static>(
        &mut self,
        #[allow(unused)] server_adapter: S,
    ) -> Result<(), ()> {
        self.server_adapter.replace(Box::new(server_adapter));
        Ok(())
    }

    fn metadata(&self) -> ClientInfo {
        ClientInfo {}
    }
}
