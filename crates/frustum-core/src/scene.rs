//! Scene definition as an immutable container for geometry and camera.

use serde::{Deserialize, Serialize};

use crate::camera::Camera;
use crate::geometry::{Mesh, PointCloud, Polyline};

/// A scene element that can be rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SceneElement {
    PointCloud(PointCloud),
    Polyline(Polyline),
    Mesh(Mesh),
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// A complete scene with camera, geometry, and explicit bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Camera for viewing the scene.
    pub camera: Camera,
    /// All geometry elements in the scene.
    pub elements: Vec<SceneElement>,
    /// Explicit scene bounds.
    pub bounds: Bounds,
}

impl Scene {
    /// Create a new scene with the given camera and bounds.
    pub fn new(camera: Camera, bounds: Bounds) -> Self {
        Self {
            camera,
            elements: Vec::new(),
            bounds,
        }
    }

    /// Add a point cloud to the scene.
    pub fn add_point_cloud(mut self, pc: PointCloud) -> Self {
        self.elements.push(SceneElement::PointCloud(pc));
        self
    }

    /// Add a polyline to the scene.
    pub fn add_polyline(mut self, line: Polyline) -> Self {
        self.elements.push(SceneElement::Polyline(line));
        self
    }

    /// Add a mesh to the scene.
    pub fn add_mesh(mut self, mesh: Mesh) -> Self {
        self.elements.push(SceneElement::Mesh(mesh));
        self
    }

    /// Serialize the scene to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a scene from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
