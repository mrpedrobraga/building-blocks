// use glam::{Mat4, UVec4};
// use image::EncodableLayout as _;
// use wgpu::util::DeviceExt as _;

// use crate::{
//     block::{
//         BlockCluster, BlockClusterGpuUniforms, BlockClusterRenderResources, RenderMaterialGpu,
//     },
//     client::application::render::{pipeline::BlocksPipeline, RenderClientGpu},
//     universe::{Universe, UniverseGpu},
// };

// impl RenderClientGpu {
//     pub fn create_universe_gpu_resources(
//         &self,
//         universe: &Universe,
//         pipeline: &BlocksPipeline,
//     ) -> UniverseGpu {
//         /* Block Palette */
//         let mut block_definitions = Vec::new();
//         for (_, (_, definition)) in universe.block_definitions.iter().enumerate() {
//             block_definitions.push(RenderMaterialGpu {
//                 atlas_position: definition.material.x_min.atlas_position,
//                 atlas_size: definition.material.x_min.atlas_size,
//             });
//         }

//         let block_definitions_buffer =
//             self.device
//                 .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                     label: Some("Universe Storage Buffer"),
//                     contents: bytemuck::cast_slice(&block_definitions),
//                     usage: wgpu::BufferUsages::STORAGE,
//                 });

//         /* Atlas Image */
//         // TODO: Use automatic packing instead of manually authored atlases?
//         let image = image::load_from_memory(include_bytes!("debug_atlas.png"))
//             .unwrap()
//             .to_rgba8();
//         let material_texture_atlas = self.device.create_texture_with_data(
//             &self.queue,
//             &wgpu::TextureDescriptor {
//                 label: Some("Universe Material Atlas"),
//                 size: wgpu::Extent3d {
//                     width: image.width(),
//                     height: image.height(),
//                     depth_or_array_layers: 1,
//                 },
//                 mip_level_count: 1,
//                 sample_count: 1,
//                 dimension: wgpu::TextureDimension::D2,
//                 format: wgpu::TextureFormat::Rgba8UnormSrgb,
//                 usage: wgpu::TextureUsages::TEXTURE_BINDING,
//                 view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
//             },
//             wgpu::wgt::TextureDataOrder::LayerMajor,
//             image.as_bytes(),
//         );
//         let atlas_view =
//             material_texture_atlas.create_view(&wgpu::TextureViewDescriptor::default());
//         let atlas_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::ClampToEdge,
//             address_mode_v: wgpu::AddressMode::ClampToEdge,
//             mag_filter: wgpu::FilterMode::Nearest,
//             min_filter: wgpu::FilterMode::Nearest,
//             mipmap_filter: wgpu::MipmapFilterMode::Nearest,
//             ..Default::default()
//         });

//         // BIND GROUP
//         let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
//             label: Some("Universe Bind Group"),
//             layout: &pipeline.universe_bind_group_layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::TextureView(&atlas_view),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: wgpu::BindingResource::Sampler(&atlas_sampler),
//                 },
//             ],
//         });

//         UniverseGpu {
//             block_definitions_buffer,
//             material_texture_atlas,
//             bind_group,
//         }
//     }

//     pub fn create_block_cluster_gpu_resources(
//         &self,
//         cluster: &BlockCluster,
//         universe_buffers: &wgpu::Buffer,
//         pipeline: &BlocksPipeline,
//     ) -> (BlockClusterGpuUniforms, BlockClusterRenderResources) {
//         // CLUSTER UNIFORM BUFFER
//         let uniforms = BlockClusterGpuUniforms {
//             transform: Mat4::from(cluster.transform).to_cols_array(),
//             size: UVec4::new(cluster.size.x, cluster.size.y, cluster.size.z, 0),
//         };
//         let uniform_buffer = self
//             .device
//             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some("Cluster Uniform Buffer"),
//                 contents: bytemuck::cast_slice(&[uniforms]),
//                 usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//             });

//         // VOXEL STORAGE
//         // Buffer with the Block Definition indices for each block of the cluster.
//         // Note that '0' is empty.
//         let block_def_indices: Vec<u32> = cluster.blocks.iter().map(|block| block.id).collect();
//         let block_buffer = self
//             .device
//             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 // TODO: Maybe ID of cluster goes here heheh...
//                 label: Some("Cluster Voxel Buffer"),
//                 contents: bytemuck::cast_slice(&block_def_indices),
//                 usage: wgpu::BufferUsages::STORAGE,
//             });

//         // BIND GROUP
//         let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
//             label: Some("Cluster Bind Group"),
//             layout: &pipeline.block_cluster_bind_group_layout,
//             entries: &[
//                 // TODO: Maybe move this to another bind group, so we bind it once per World?
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: self.global_uniform_buffer.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: uniform_buffer.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 2,
//                     resource: universe_buffers.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 3,
//                     resource: block_buffer.as_entire_binding(),
//                 },
//             ],
//         });

//         (
//             uniforms,
//             BlockClusterRenderResources {
//                 block_buffer,
//                 uniform_buffer,
//                 bind_group,
//                 num_voxels: cluster.size.x * cluster.size.y * cluster.size.z,
//             },
//         )
//     }
// }
