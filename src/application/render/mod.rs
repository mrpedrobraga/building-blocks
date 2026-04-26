//! # Render
//!
//! Render engine that renders everything to the screen :-)

use std::sync::Arc;

use glam::{Mat4, UVec4};
use tracing::info_span;
use wgpu::{
    include_wgsl, util::DeviceExt, Adapter, BindGroupLayout, BindGroupLayoutEntry, Device,
    Instance, InstanceDescriptor, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassDepthStencilAttachment, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages, TextureView,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    block::{
        BlockCluster, BlockClusterGpuUniforms, BlockClusterRenderResources, RenderMaterialGpu,
    },
    universe::{Universe, UniverseGpu, World},
};

pub struct RenderState {
    device: Device,
    queue: Queue,
    target: RenderTarget,
    pipeline: BlocksPipeline,
    pub window: Arc<Window>,
    uniforms: GlobalUniforms,
    global_uniform_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub view_projection: [f32; 16],
}

pub struct RenderTarget {
    surface: Surface<'static>,
    depth_texture_view: TextureView,
    surface_config: SurfaceConfiguration,
    surface_size: PhysicalSize<u32>,
}

pub struct BlocksPipeline {
    render_pipeline: RenderPipeline,
    cluster_bind_group_layout: BindGroupLayout,
}

pub fn create_projection_matrix(width: u32, height: u32) -> Mat4 {
    let aspect_ratio = width as f32 / height as f32;
    let fov_y = 45.0_f32.to_radians(); // Field of View
    let z_near = 0.1; // Closest render distance
    let z_far = 100.0; // Furthest render distance

    // wgpu uses a depth range of [0, 1].
    // Mat4::perspective_lh is for Left-Handed coordinates (common in voxel engines)
    // There is also Mat4::perspective_rh if you prefer Right-Handed.
    let perspective = Mat4::perspective_lh(fov_y, aspect_ratio, z_near, z_far);
    let view = glam::Mat4::look_at_lh(
        glam::Vec3::new(5.0, 5.0, 5.0), // Camera position
        glam::Vec3::ZERO,               // Looking at center
        glam::Vec3::Y,                  // Up direction
    );

    perspective * view
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let target = RenderTarget::new(window.inner_size(), surface, &device, &adapter);
        let pipeline = BlocksPipeline::new(&device, &target);

        let uniforms = GlobalUniforms {
            view_projection: create_projection_matrix(
                target.surface_config.width,
                target.surface_config.height,
            )
            .to_cols_array(),
        };

        let global_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cluster Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        RenderState {
            device,
            queue,
            pipeline,
            target,
            window,
            uniforms,
            global_uniform_buffer,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.target.configure(&self.device, new_size);
        self.update_uniforms();
    }

    pub fn create_universe_buffers(&self, universe: &Universe) -> wgpu::Buffer {
        let mut block_definitions = Vec::new();
        for (_, (_, definition)) in universe.block_definitions.iter().enumerate() {
            block_definitions.push(RenderMaterialGpu {
                color: definition.material.x_min.color,
            });
        }

        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Universe Storage Buffer"),
                contents: bytemuck::cast_slice(&block_definitions),
                usage: wgpu::BufferUsages::STORAGE,
            })
    }

    pub fn create_cluster_resources(
        &self,
        cluster: &BlockCluster,
        universe_buffers: &wgpu::Buffer,
        pipeline: &BlocksPipeline,
    ) -> (BlockClusterGpuUniforms, BlockClusterRenderResources) {
        // CLUSTER UNIFORM BUFFER
        let uniforms = BlockClusterGpuUniforms {
            transform: Mat4::from(cluster.transform).to_cols_array(),
            size: UVec4::new(cluster.size.x, cluster.size.y, cluster.size.z, 0),
        };
        let uniform_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cluster Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // VOXEL STORAGE
        // Buffer with the Block Definition indices for each block of the cluster.
        // Note that '0' is empty.
        let block_def_indices: Vec<u32> = cluster.blocks.iter().map(|block| block.id).collect();
        let block_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                // TODO: Maybe ID of cluster goes here heheh...
                label: Some("Cluster Voxel Buffer"),
                contents: bytemuck::cast_slice(&block_def_indices),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // BIND GROUP
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cluster Bind Group"),
            layout: &pipeline.cluster_bind_group_layout,
            entries: &[
                // TODO: Maybe move this to another bind group, so we bind it once per World?
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.global_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: universe_buffers.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: block_buffer.as_entire_binding(),
                },
            ],
        });

        (
            uniforms,
            BlockClusterRenderResources {
                block_buffer,
                uniform_buffer,
                bind_group,
                num_voxels: cluster.size.x * cluster.size.y * cluster.size.z,
            },
        )
    }

    pub fn update_uniforms(&mut self) {
        self.uniforms = GlobalUniforms {
            view_projection: create_projection_matrix(
                self.target.surface_config.width,
                self.target.surface_config.height,
            )
            .to_cols_array(),
        };

        self.queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn prepare(&mut self, universe: &mut Universe, world: &mut World) {
        self.update_uniforms();

        if universe.gpu.is_none() {
            universe.gpu = Some(UniverseGpu {
                block_definitions_buffer: Self::create_universe_buffers(&self, universe),
            })
        }

        for (_, block_cluster) in world.block_clusters.iter_mut() {
            if block_cluster.gpu.is_none() {
                block_cluster.gpu = Some(Self::create_cluster_resources(
                    &self,
                    block_cluster,
                    universe
                        .gpu
                        .as_ref()
                        .map(|x| &x.block_definitions_buffer)
                        .unwrap(),
                    &self.pipeline,
                    // TODO: Move this out of here into a different set of uniforms
                    // that is global and not per-cluster!
                ));
            }
        }
    }

    pub fn render(&mut self, world: &World) {
        let s = info_span!("render");
        let _ = s.enter();

        let output = self.target.surface.get_current_texture();
        let wgpu::CurrentSurfaceTexture::Success(output) = output else {
            return;
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.9,
                            g: 0.9,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.target.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline.render_pipeline);
            for (_, cluster) in world.block_clusters.iter() {
                if let Some((_, resources)) = &cluster.gpu {
                    render_pass.set_bind_group(0, &resources.bind_group, &[]);
                    render_pass.draw(0..36, 0..resources.num_voxels);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl RenderTarget {
    pub fn new(
        size: PhysicalSize<u32>,
        surface: Surface<'static>,
        device: &Device,
        adapter: &Adapter,
    ) -> Self {
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            view_formats: vec![surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(&device, &surface_config);

        let depth_texture_view = Self::create_depth_texture(device, &surface_config);

        RenderTarget {
            surface,
            depth_texture_view,
            surface_config,
            surface_size: size,
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        color_texture_config: &SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: color_texture_config.width,
            height: color_texture_config.height,
            depth_or_array_layers: 1,
        };
        let descriptor = wgpu::TextureDescriptor {
            label: Some("Render Target Depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&descriptor);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn configure(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        self.surface_size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&device, &self.surface_config);
        self.depth_texture_view = Self::create_depth_texture(device, &self.surface_config);
    }
}

impl BlocksPipeline {
    pub fn new(device: &Device, target: &RenderTarget) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let cluster_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Block Bind Group Layout"),
                entries: &[
                    // Binding 0: Global Uniforms!
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Binding 1: Block cluster Uniforms!
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Binding 2: Universe's Block Definition buffer.
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Binding 3: The current cluster's block data.
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout Descriptor"),
            bind_group_layouts: &[Some(&cluster_bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target.surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        BlocksPipeline {
            render_pipeline,
            cluster_bind_group_layout,
        }
    }
}
