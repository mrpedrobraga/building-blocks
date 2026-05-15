use core::f32;
use std::ops::{Mul};

use glam::{Mat4, Quat, UVec2, Vec3, Vec4, vec3};
use wgpu::util::DeviceExt;

use super::{gpu::Gpu, pipeline::voxels::WorldUniforms, *};
use crate::{
    server::messages::{ServerSceneMessage, ServerUniverseMessage, ServerWorldMessage},
    world::{
        block::RenderMaterial,
        camera::{Camera, CameraProjection},
    },
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

        let bind_group_layout = UniverseRenderState::bind_group_layout(gpu);

        let block_appearance_palette = DashMap::new();
        let block_appearance_palette_gpu =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Universe Block Appearances Buffer"),
                    contents: bytemuck::cast_slice(&[
                        // TODO: Preinitialize some amount here, but not sure how much?
                        // maybe the server will know and tell me!
                        RenderMaterial {
                            atlas_position: Vec2::new(0.0, 0.0),
                            atlas_size: Vec2::new(8.0, 8.0),
                        },
                        RenderMaterial {
                            atlas_position: Vec2::new(8.0, 0.0),
                            atlas_size: Vec2::new(8.0, 8.0),
                        }
                    ]),
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
            layout: &bind_group_layout,
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
            block_appearance_palette,
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
        let bind_group_layout = WorldRenderState::bind_group_layout(gpu);

        let current_scene = CurrentSceneRenderState::new(gpu);
        let layout_cache = DashMap::new();

        let block_group = &current_scene.root_layout.block_groups[0];
        let block_group_size = block_group.uniforms.size.as_vec3();

        let cam = camera_orbit(block_group_size, 0.0);
        let view_matrix = cam.view_matrix(UVec2::new(640, 640));

        let uniforms = WorldUniforms {
            view_matrix: view_matrix.to_cols_array(),
            camera_world_position: Vec4::new(0.0, 0.0, 0.0, 1.0),
            global_time: 0.0,
            _padding: [0.0, 0.0, 0.0],
        };

        let uniforms_gpu = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("World Uniforms Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_gpu.as_entire_binding(),
            }],
        });

        Self {
            current_scene,
            layout_cache,
            uniforms,
            uniforms_gpu,
            bind_group,
            beggining: Instant::now(),
        }
    }

    pub fn tick(&mut self, gpu: &Gpu, screen_size: UVec2) {
        let block_group_size = self.current_scene.root_layout.block_groups[0]
            .uniforms
            .size
            .as_vec3();

        let cam = camera_orbit(
            block_group_size,
            self.beggining.elapsed().as_secs_f32(),
        );
        let view_matrix = cam.view_matrix(screen_size);

        let uniforms = WorldUniforms {
            view_matrix: view_matrix.to_cols_array(),
            camera_world_position: Vec4::new(cam.position.x, cam.position.y, cam.position.z, 1.0),
            global_time: self.beggining.elapsed().as_secs_f32(),
            _padding: [0.0, 0.0, 0.0],
        };

        gpu.queue
            .write_buffer(&self.uniforms_gpu, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn patch(&mut self, gpu: &Gpu, message: &ServerWorldMessage) {
        match message {
            ServerWorldMessage::EnterWorld { .. } | ServerWorldMessage::LeaveWorld => {
                self.current_scene = CurrentSceneRenderState::new(gpu);
                self.layout_cache = DashMap::new();
            }
            ServerWorldMessage::CurrentScene(server_scene_message) => {
                self.current_scene.patch(gpu, server_scene_message)
            }
        }
    }
}

fn camera_orbit(_block_group_size: Vec3, _time: f32) -> Camera {
    let distance = 50.0;
    let mut cam = Camera::new(
        vec3(distance, distance, distance * 0.5).rotate_z(_time * 0.125 * f32::consts::TAU).mul(1.0),
        Quat::default(),
        CameraProjection::Perspective {
            vertical_fov_radians: 60.0_f32.to_radians(),
            z_near_clipping_plane: 0.1,
            z_far_clipping_plane: 10000.0,
        },
    );
    cam.look_at(Vec3::ZERO, Vec3::Z);
    cam
}

impl CurrentSceneRenderState {
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            environment: EnvironmentRenderState {},
            root_layout: LayoutRenderState::example(gpu),
        }
    }

    pub fn patch(&mut self, gpu: &Gpu, message: &ServerSceneMessage) {
        match message {
            ServerSceneMessage::EnterScene { .. } | ServerSceneMessage::LeaveScene {} => {
                self.environment = EnvironmentRenderState {};
                self.root_layout = LayoutRenderState::example(gpu);
            }
            ServerSceneMessage::BlockGroup(_server_block_group_message) => {
                // TODO: Handle this I guess?
            }
        }
    }
}

impl LayoutRenderState {
    pub fn new() -> Self {
        let block_groups = Vec::new();
        let sub_layouts = Vec::new();

        Self {
            block_groups,
            sub_layouts,
        }
    }

    pub fn example(gpu: &Gpu) -> Self {
        let sub_layouts = Vec::new();

        let block_groups = vec![
            BlockGroupRenderState::example(gpu, 1),
            BlockGroupRenderState::example(gpu, 2),
        ];

        Self {
            block_groups,
            sub_layouts,
        }
    }
}

impl Default for LayoutRenderState {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockGroupRenderState {
    pub fn new(gpu: &Gpu) -> Self {
        let uniforms = BlockGroupUniforms {
            transform: Mat4::IDENTITY.to_cols_array(),
            inv_transform: Mat4::IDENTITY.to_cols_array(),
            size: UVec3::new(0, 0, 0),
            _padding: 0,
        };
        let uniforms_gpu = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Block Group Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        // TODO: Preallocate more space.
        let block_appearance_data = Vec::new();
        let block_appearance_data_gpu =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Block Group Nametable"),
                    contents: bytemuck::cast_slice(block_appearance_data.as_slice()),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        // TODO: Store this...
        let bind_group_layout = Self::bind_group_layout(gpu);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Block Group Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_gpu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: block_appearance_data_gpu.as_entire_binding(),
                },
            ],
        });

        Self {
            uniforms,
            uniforms_gpu,
            block_appearance_data,
            block_appearance_data_gpu,
            bind_group,
        }
    }

    pub fn example(gpu: &Gpu, material: u32) -> Self {
        
        let block_group_size = UVec3::new(100, 100, 21);
        //let block_group_half_size = block_group_size.div(UVec3::new(2, 2, 2)).as_vec3();
        let mut transform = Mat4::from_translation(block_group_size.as_vec3() * vec3(-0.5, -0.5, 0.0));

        if material == 2 {
            transform = Mat4::from_translation(block_group_size.as_vec3() * vec3(-1.0, -1.0, 0.2));
        }

        let uniforms = BlockGroupUniforms {
            transform: transform.to_cols_array(),
            inv_transform: transform.inverse().to_cols_array(),
            size: block_group_size,
            _padding: 0,
        };
        let uniforms_gpu = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Block Group Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        // TODO: Preallocate more space.
        let blocks_iter = (0..block_group_size.z)
            .flat_map(|z| (0..block_group_size.y).map(move |y| (z, y)))
            .flat_map(move |(z, y)| (0..block_group_size.x).map(move |x| (z, y, x)));
        let block_appearance_data = blocks_iter
            .map(|(z, y, x)| {
                let point = Vec3::new(x as f32, y as f32, z as f32);
                //let put_block = point.distance_squared(block_group_half_size) < block_group_half_size.x.powf(2.0);
                let put_block = point.z < 10.0 + 5.0 * point.x.mul(0.1).sin() + 5.0 * point.y.mul(0.1).cos();

                if put_block
                {
                    BlockAppearanceEntry { idx_in_palette: material }
                } else {
                    BlockAppearanceEntry { idx_in_palette: 0 }
                }
            })
            .collect::<Vec<_>>();

        let block_appearance_data_gpu =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Block Group Nametable"),
                    contents: bytemuck::cast_slice(block_appearance_data.as_slice()),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        // TODO: Store this...
        let bind_group_layout = Self::bind_group_layout(gpu);
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Block Group Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms_gpu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: block_appearance_data_gpu.as_entire_binding(),
                },
            ],
        });

        Self {
            uniforms,
            uniforms_gpu,
            block_appearance_data,
            block_appearance_data_gpu,
            bind_group,
        }
    }
}
