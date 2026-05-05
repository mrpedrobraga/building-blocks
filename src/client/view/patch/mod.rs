use super::*;
use crate::server::messages::{ServerUniverseMessage, ServerWorldMessage};

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
}

impl WorldView {
    pub fn patch(&mut self, message: &ServerWorldMessage) {
        match message {
            ServerWorldMessage::EnterWorld { .. } => self.current_scene = SceneView::new(),
            ServerWorldMessage::LeaveWorld => self.current_scene = SceneView::new(),
        }
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
}

impl LayoutView {
    pub fn new() -> Self {
        Self {
            block_groups: Vec::new(),
            sub_layouts: Vec::new(),
        }
    }
}
