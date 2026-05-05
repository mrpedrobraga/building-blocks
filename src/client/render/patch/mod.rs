use super::*;
use crate::server::messages::{ServerSceneMessage, ServerUniverseMessage, ServerWorldMessage};

impl GameRenderState {
    pub fn new() -> Self {
        Self {
            universe_state: UniverseRenderState::new(),
            world_state: WorldRenderState::new(),
        }
    }
}

impl Default for GameRenderState {
    fn default() -> Self {
        Self::new()
    }
}

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

impl Default for UniverseRenderState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldRenderState {
    pub fn new() -> Self {
        Self {
            current_scene: CurrentSceneRenderState::new(),
            layout_cache: DashMap::new(),
        }
    }

    pub fn patch(&mut self, message: &ServerWorldMessage) {
        match message {
            ServerWorldMessage::EnterWorld { .. } | ServerWorldMessage::LeaveWorld => {
                self.current_scene = CurrentSceneRenderState::new();
                self.layout_cache = DashMap::new();
            }
            ServerWorldMessage::CurrentScene(server_scene_message) => {
                self.current_scene.patch(server_scene_message)
            }
        }
    }
}

impl Default for WorldRenderState {
    fn default() -> Self {
        Self::new()
    }
}

impl CurrentSceneRenderState {
    pub fn new() -> Self {
        Self {
            environment: EnvironmentRenderState {},
            root_layout: LayoutRenderState::new(),
        }
    }

    pub fn patch(&mut self, message: &ServerSceneMessage) {
        match message {
            ServerSceneMessage::EnterScene { .. } | ServerSceneMessage::LeaveScene {} => {
                self.environment = EnvironmentRenderState {};
                self.root_layout = LayoutRenderState::new();
            }
            ServerSceneMessage::BlockGroup(_server_block_group_message) => {
                // TODO: Handle this I guess?
            }
        }
    }
}

impl Default for CurrentSceneRenderState {
    fn default() -> Self {
        Self::new()
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

impl Default for LayoutRenderState {
    fn default() -> Self {
        Self::new()
    }
}
