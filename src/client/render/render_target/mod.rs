//! # Render Target
//!
//! Trait and structs for things that can be rendered to. [RenderTarget] is the main trait of this module,
//! and the trait that all render targets implement.

use wgpu::{SurfaceTexture, TextureView};

pub mod window;

/// See the module-level documentation.
pub trait RenderTarget {
    /// Returns a [TextureView] that a render pass can render to!
    fn texture_view_set(&self) -> Result<TextureViewSet, ()>;
}

/// A set of textures to which render passes can draw.
pub struct TextureViewSet {
    pub surface: Option<SurfaceTexture>,
    pub albedo: TextureView,
    pub depth: TextureView,
}
