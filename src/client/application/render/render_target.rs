//! # Render Target
//!
//! Trait and structs for things that can be rendered to. [RenderTarget] is the main trait of this module,
//! and the trait that all render targets implement.

use wgpu::{Device, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureView};
use winit::dpi::PhysicalSize;

/// See the module-level documentation.
pub trait RenderTarget {
    /// Returns a [TextureView] that a render pass can render to!
    fn texture_view(&self) -> Result<TextureView, ()>;
}

/// This game is famously known for having things be drawn to the window!
pub struct WindowRenderTarget {
    pub(crate) surface: Surface<'static>,
    pub(crate) depth_texture_view: TextureView,
    pub(crate) surface_config: SurfaceConfiguration,
    pub surface_size: PhysicalSize<u32>,
}

impl WindowRenderTarget {
    pub fn new(
        size: PhysicalSize<u32>,
        surface: Surface<'static>,
        format: TextureFormat,
        device: &Device,
    ) -> Self {
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            view_formats: vec![format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: size.width,
            height: size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        surface.configure(&device, &surface_config);

        let depth_texture_view = Self::create_depth_texture(device, &surface_config);

        WindowRenderTarget {
            surface,
            depth_texture_view,
            surface_config,
            surface_size: size,
        }
    }

    pub fn configure(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        self.surface_size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&device, &self.surface_config);
        self.depth_texture_view = Self::create_depth_texture(device, &self.surface_config);
    }

    fn create_depth_texture(
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

    pub fn resize(&mut self, gpu_device: &wgpu::Device, new_size: PhysicalSize<u32>) {
        self.configure(gpu_device, new_size);
    }
}

impl RenderTarget for WindowRenderTarget {
    fn texture_view(&self) -> Result<TextureView, ()> {
        let current_texture = self.surface.get_current_texture();
        let wgpu::CurrentSurfaceTexture::Success(current_texture) = current_texture else {
            return Err(());
        };
        Ok(current_texture.texture.create_view(&Default::default()))
    }
}
