//! Camera definition with explicit parameters and matrix generation.

use glam::{Mat4, Vec3};
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
    pub fn perspective(position: [f32; 3], target: [f32; 3], fov_degrees: f32) -> Self {
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
    pub fn orthographic(position: [f32; 3], target: [f32; 3], view_height: f32) -> Self {
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

    /// Compute the view matrix (world → camera space).
    pub fn view_matrix(&self) -> Mat4 {
        let eye = Vec3::from_array(self.position);
        let target = Vec3::from_array(self.target);
        let up = Vec3::from_array(self.up);
        Mat4::look_at_rh(eye, target, up)
    }

    /// Compute the projection matrix for the given aspect ratio.
    ///
    /// Uses wgpu/Vulkan NDC convention:
    /// - Right-handed coordinate system
    /// - Y-up in clip space
    /// - Z in [0, 1] (not [-1, 1] like OpenGL)
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective => {
                let fov_radians = self.fov_or_height.to_radians();
                // perspective_rh uses Vulkan/wgpu NDC (Z in [0, 1])
                Mat4::perspective_rh(fov_radians, aspect_ratio, self.near, self.far)
            }
            Projection::Orthographic => {
                let half_height = self.fov_or_height / 2.0;
                let half_width = half_height * aspect_ratio;
                // orthographic_rh uses Vulkan/wgpu NDC (Z in [0, 1])
                Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    self.near,
                    self.far,
                )
            }
        }
    }

    /// Compute the combined view-projection matrix.
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perspective_camera_matrices() {
        let camera = Camera::perspective([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 45.0);
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(1.0);
        let vp = camera.view_projection_matrix(1.0);

        // View matrix should transform camera position to origin
        let cam_pos = Vec3::new(0.0, 0.0, 5.0);
        let transformed = view.transform_point3(cam_pos);
        assert!((transformed.z - 0.0).abs() < 1e-5, "Camera should be at origin in view space");

        // VP matrix should be product of projection and view
        let expected_vp = proj * view;
        assert_eq!(vp, expected_vp);
    }

    #[test]
    fn test_orthographic_camera_matrices() {
        let camera = Camera::orthographic([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 10.0);
        let proj = camera.projection_matrix(1.0);

        // Orthographic projection should not have perspective division
        // Points at different Z should project to same XY
        let p1 = proj.project_point3(Vec3::new(1.0, 1.0, -1.0));
        let p2 = proj.project_point3(Vec3::new(1.0, 1.0, -2.0));
        assert!((p1.x - p2.x).abs() < 1e-5);
        assert!((p1.y - p2.y).abs() < 1e-5);
    }
}
