//! Geometry primitives: point clouds, polylines, and triangle meshes.

use serde::{Deserialize, Serialize};

/// A point cloud with per-point positions and optional scalar values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloud {
    /// Flattened array of vertex positions [x0, y0, z0, x1, y1, z1, ...].
    pub positions: Vec<f32>,
    /// Optional per-point scalar values for colormap mapping.
    pub scalars: Option<Vec<f32>>,
    /// Uniform point size in pixels.
    pub point_size: f32,
}

impl PointCloud {
    /// Create a new point cloud from positions.
    pub fn new(positions: Vec<f32>, point_size: f32) -> Self {
        Self {
            positions,
            scalars: None,
            point_size,
        }
    }

    /// Set scalar values for colormap mapping.
    pub fn with_scalars(mut self, scalars: Vec<f32>) -> Self {
        self.scalars = Some(scalars);
        self
    }

    /// Number of points in the cloud.
    pub fn len(&self) -> usize {
        self.positions.len() / 3
    }

    /// Check if point cloud is empty.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// A polyline defined by a sequence of vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline {
    /// Flattened array of vertex positions [x0, y0, z0, x1, y1, z1, ...].
    pub positions: Vec<f32>,
    /// Uniform line width in pixels.
    pub line_width: f32,
}

impl Polyline {
    /// Create a new polyline from positions.
    pub fn new(positions: Vec<f32>, line_width: f32) -> Self {
        Self {
            positions,
            line_width,
        }
    }
}

/// An indexed triangle mesh with optional normals and scalar values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    /// Flattened array of vertex positions [x0, y0, z0, x1, y1, z1, ...].
    pub positions: Vec<f32>,
    /// Triangle indices (3 indices per triangle).
    pub indices: Vec<u32>,
    /// Optional per-vertex normals [nx0, ny0, nz0, ...].
    pub normals: Option<Vec<f32>>,
    /// Optional per-vertex scalar values for colormap mapping.
    pub scalars: Option<Vec<f32>>,
}

impl Mesh {
    /// Create a new mesh from positions and indices.
    pub fn new(positions: Vec<f32>, indices: Vec<u32>) -> Self {
        Self {
            positions,
            indices,
            normals: None,
            scalars: None,
        }
    }

    /// Set vertex normals.
    pub fn with_normals(mut self, normals: Vec<f32>) -> Self {
        self.normals = Some(normals);
        self
    }

    /// Set scalar values for colormap mapping.
    pub fn with_scalars(mut self, scalars: Vec<f32>) -> Self {
        self.scalars = Some(scalars);
        self
    }

    /// Number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}
