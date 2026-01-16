//! Frustum Render
//!
//! GPU rendering backend for Frustum using wgpu.

use frustum_core::Scene;
use thiserror::Error;

/// Errors that can occur during rendering.
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to create GPU adapter")]
    AdapterCreation,
    #[error("Failed to create GPU device: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),
    #[error("Failed to encode PNG: {0}")]
    PngEncoding(#[from] image::ImageError),
}

/// Render configuration.
pub struct RenderConfig {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Background color as RGBA (0.0 to 1.0).
    pub background: [f32; 4],
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            background: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Render a scene to a PNG image.
///
/// This is the primary entry point for headless rendering.
pub fn render_to_png(_scene: &Scene, _config: &RenderConfig) -> Result<Vec<u8>, RenderError> {
    // TODO: Implement wgpu rendering pipeline
    // This is a placeholder that will be implemented with:
    // 1. GPU adapter/device creation
    // 2. Shader compilation
    // 3. Pipeline setup
    // 4. Geometry upload
    // 5. Render pass execution
    // 6. Buffer readback
    // 7. PNG encoding

    todo!("Rendering pipeline not yet implemented")
}
