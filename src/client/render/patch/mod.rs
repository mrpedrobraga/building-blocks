use glam::{Quat, UVec2, Vec3};
use wgpu::util::DeviceExt;

use super::{gpu::Gpu, pipeline::voxels::WorldUniforms, *};
use crate::{
    server::messages::{ServerSceneMessage, ServerUniverseMessage, ServerWorldMessage},
    world::{block::RenderMaterial, camera::Camera},
};

impl GameRenderState {
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            universe_state: UniverseRenderState::new(gpu),
            world_state: WorldRenderState::new(gpu),
        }
    }
}

impl UniverseRenderState {
    pub fn new(gpu: &Gpu) -> Self {
        use image::EncodableLayout as _;
        use wgpu::util::DeviceExt as _;

        let layout = UniverseRenderState::bind_group_layout(gpu);

        let block_appearance_palette_gpu =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Universe Block Appearances Buffer"),
                    contents: bytemuck::cast_slice(&[RenderMaterial {
                        atlas_position: Vec2::new(0.0, 0.0),
                        atlas_size: Vec2::new(8.0, 8.0),
                    }]),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        let texture_atlas =
            image::open("examples/example_universe/packs/toyvox-default/textures/main_atlas.png")
                .unwrap()
                .to_rgba8();
        let texture_atlas_descriptor = wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: texture_atlas.width(),
                height: texture_atlas.height(),
                // TODO: Yeah this here would allow us to
                // have multiple atlases :-)
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture_atlas_gpu = gpu.device.create_texture_with_data(
            &gpu.queue,
            &texture_atlas_descriptor,
            wgpu::wgt::TextureDataOrder::LayerMajor,
            texture_atlas.as_bytes(),
        );

        let texture_atlas_view = texture_atlas_gpu.create_view(&Default::default());
        let texture_atlas_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Universe Bind Group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: block_appearance_palette_gpu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas_sampler),
                },
            ],
        });

        Self {
            block_appearance_palette: DashMap::new(),
            block_appearance_palette_gpu,
            texture_atlas,
            texture_atlas_gpu,
            bind_group,
        }
    }

    pub fn patch(&mut self, message: &ServerUniverseMessage) {
        match message {
            ServerUniverseMessage::Let { .. } => self.block_appearance_palette = DashMap::new(),
        }
    }
}

impl WorldRenderState {
    pub fn new(gpu: &Gpu) -> Self {
        let layout = WorldRenderState::bind_grop_layout(gpu);

        let view_matrix = {
            let mut cam = Camera::new(
                Vec3::new(10.0, 10.0, 10.0),
                Quat::default(),
                crate::world::camera::CameraProjection::Perspective {
                    vertical_fov_radians: 60.0,
                    z_near_clipping_plane: 0.01,
                    z_far_clipping_plane: 100.0,
                },
            );
            cam.look_at(Vec3::new(1.5, 1.5, 1.5), Vec3::Z);
            cam.view_matrix(UVec2::new(640, 640)).to_cols_array()
        };

        let uniforms = WorldUniforms {
            view_matrix,
            global_time: 0.0,
            _padding: [0.0, 0.0, 0.0],
        };

        let uniforms_gpu = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("World Uniforms Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_gpu.as_entire_binding(),
            }],
        });

        Self {
            current_scene: CurrentSceneRenderState::new(),
            layout_cache: DashMap::new(),
            uniforms,
            uniforms_gpu,
            bind_group,
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
