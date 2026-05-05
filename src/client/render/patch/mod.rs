use super::*;
use crate::server::messages::{ServerUniverseMessage, ServerWorldMessage};

impl UniverseRenderState {
    pub fn new() -> Self {
        Self {
            block_appearance_palette: DashMap::new(),
        }
    }

    pub fn patch(&mut self, message: &ServerUniverseMessage) {
        match message {
            ServerUniverseMessage::Let { .. } => self.block_appearance_palette = DashMap::new(),
        }
    }
}

impl WorldRenderState {
    pub fn patch(&mut self, message: &ServerWorldMessage) {
        match message {
            ServerWorldMessage::EnterWorld { .. } => {
                self.current_scene = CurrentSceneRenderState::new()
            }
            ServerWorldMessage::LeaveWorld => self.current_scene = CurrentSceneRenderState::new(),
        }
    }
}

impl CurrentSceneRenderState {
    pub fn new() -> Self {
        Self {
            environment: EnvironmentRenderState {},
            root_layout: LayoutRenderState::new(),
        }
    }
}

impl LayoutRenderState {
    pub fn new() -> Self {
        Self {
            block_groups: Vec::new(),
            sub_layouts: Vec::new(),
        }
    }
}
