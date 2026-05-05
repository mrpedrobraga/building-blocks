use tracing::info;

use super::*;
use crate::server::messages::{ServerSceneMessage, ServerUniverseMessage, ServerWorldMessage};

impl GameView {
    pub fn new() -> Self {
        GameView {
            current_universe: UniverseView::new(),
            current_world: WorldView::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_universe.reset();
        self.current_world.reset();
    }
}

impl Default for GameView {
    fn default() -> Self {
        Self::new()
    }
}

impl UniverseView {
    pub fn new() -> Self {
        Self {
            block_types_list: DashMap::new(),
        }
    }

    pub fn patch(&mut self, message: &ServerUniverseMessage) {
        match message {
            ServerUniverseMessage::Let { .. } => self.block_types_list = DashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.block_types_list = DashMap::new();
    }
}

impl Default for UniverseView {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldView {
    pub fn new() -> Self {
        WorldView {
            current_scene: SceneView::new(),
            layout_views: DashMap::new(),
        }
    }

    pub fn patch(&mut self, message: &ServerWorldMessage) {
        match message {
            ServerWorldMessage::EnterWorld { .. } | ServerWorldMessage::LeaveWorld => {
                info!("[Client] Entering new world...");
                self.current_scene = SceneView::new();
                self.layout_views = DashMap::new()
            }
            ServerWorldMessage::CurrentScene(scene_message) => {
                self.current_scene.patch(scene_message)
            }
        }
    }

    pub fn reset(&mut self) {
        self.current_scene.reset();
        self.layout_views = DashMap::new();
    }
}

impl Default for WorldView {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneView {
    pub fn new() -> Self {
        Self {
            environment: EnvironmentView {
                xy_plane_is_solid_floor: true,
            },
            root_layout: LayoutView::new(),
        }
    }

    pub fn patch(&mut self, message: &ServerSceneMessage) {
        match message {
            ServerSceneMessage::EnterScene { .. } | ServerSceneMessage::LeaveScene {} => {
                self.environment = EnvironmentView {
                    xy_plane_is_solid_floor: false,
                };
                self.root_layout = LayoutView::new();
            }
            ServerSceneMessage::BlockGroup(_server_block_group_message) => {
                // TOOD: Do something?
            }
        }
    }

    pub fn reset(&mut self) {
        self.environment = EnvironmentView {
            xy_plane_is_solid_floor: true,
        };
        self.root_layout = LayoutView::new();
    }
}

impl Default for SceneView {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutView {
    pub fn new() -> Self {
        Self {
            block_groups: Vec::new(),
            sub_layouts: Vec::new(),
        }
    }
}

impl Default for LayoutView {
    fn default() -> Self {
        Self::new()
    }
}
