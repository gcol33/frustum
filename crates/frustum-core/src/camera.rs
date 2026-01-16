//! Camera definition with explicit parameters.

use serde::{Deserialize, Serialize};

/// Projection type for the camera.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Projection {
    Perspective,
    Orthographic,
}

/// Camera with explicit position, target, and projection parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// Camera position in world coordinates.
    pub position: [f32; 3],
    /// Point the camera is looking at.
    pub target: [f32; 3],
    /// Up vector for camera orientation.
    pub up: [f32; 3],
    /// Projection type.
    pub projection: Projection,
    /// Near clipping plane distance.
    pub near: f32,
    /// Far clipping plane distance.
    pub far: f32,
    /// Field of view in degrees (perspective) or view height (orthographic).
    pub fov_or_height: f32,
}

impl Camera {
    /// Create a new perspective camera.
    pub fn perspective(
        position: [f32; 3],
        target: [f32; 3],
        fov_degrees: f32,
    ) -> Self {
        Self {
            position,
            target,
            up: [0.0, 1.0, 0.0],
            projection: Projection::Perspective,
            near: 0.1,
            far: 1000.0,
            fov_or_height: fov_degrees,
        }
    }

    /// Create a new orthographic camera.
    pub fn orthographic(
        position: [f32; 3],
        target: [f32; 3],
        view_height: f32,
    ) -> Self {
        Self {
            position,
            target,
            up: [0.0, 1.0, 0.0],
            projection: Projection::Orthographic,
            near: 0.1,
            far: 1000.0,
            fov_or_height: view_height,
        }
    }
}
