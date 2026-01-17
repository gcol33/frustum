//! Figure Audit Bundle for sanity checking and regression testing.
//!
//! This module provides structured evidence for AI-assisted validation.
//! The audit bundle contains metadata, geometry probes, and image-derived
//! metrics that allow checking invariants without evaluating raw pixels.

use serde::{Deserialize, Serialize};

/// Complete audit bundle emitted alongside a render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBundle {
    /// Structural metadata about the render.
    pub metadata: RenderMetadata,
    /// Numeric geometry probes computed during rendering.
    pub geometry: GeometryProbes,
    /// Image-derived summary metrics.
    pub image_metrics: ImageMetrics,
    /// Results of invariant checks.
    pub invariants: InvariantResults,
}

/// Structural metadata about the render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetadata {
    /// Hash of the scene JSON for identity checking.
    pub scene_hash: String,
    /// Frustum schema version.
    pub schema_version: String,
    /// Renderer version.
    pub renderer_version: String,
    /// GPU backend used (Vulkan, Metal, DX12, etc.).
    pub backend: String,
    /// GPU adapter name.
    pub adapter: String,
    /// Output resolution.
    pub resolution: [u32; 2],
    /// Camera parameters summary.
    pub camera: CameraSummary,
    /// World bounds from scene.
    pub world_bounds: BoundsSummary,
    /// Count of primitives by type.
    pub primitive_counts: PrimitiveCounts,
}

/// Camera parameters summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSummary {
    pub projection: String,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub near: f32,
    pub far: f32,
    pub fov_or_height: f32,
}

/// Bounds summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsSummary {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub center: [f32; 3],
    pub extent: [f32; 3],
}

/// Primitive counts by type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimitiveCounts {
    pub meshes: u32,
    pub total_triangles: u32,
    pub total_vertices: u32,
    pub point_clouds: u32,
    pub total_points: u32,
    pub polylines: u32,
    pub total_line_segments: u32,
}

/// Numeric geometry probes computed during rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryProbes {
    /// Bounding box of rendered geometry in NDC space.
    pub ndc_bounds: Option<BoundsSummary>,
    /// Depth range statistics.
    pub depth_stats: DepthStats,
    /// Number of degenerate primitives (zero-area triangles, etc.).
    pub degenerate_count: u32,
    /// Number of primitives clipped by near/far planes.
    pub clipped_count: u32,
    /// Number of back-facing triangles (culled).
    pub backface_count: u32,
    /// Whether any geometry intersects the view frustum.
    pub geometry_visible: bool,
    /// Whether any NaN or Inf values were detected.
    pub has_invalid_values: bool,
}

/// Depth buffer statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    /// Percentage of pixels at far plane (nothing rendered).
    pub far_plane_percentage: f32,
}

/// Image-derived summary metrics (computed from rendered pixels).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetrics {
    /// Color histogram (16 bins per channel).
    pub histogram: ColorHistogram,
    /// Edge density metric (0.0 to 1.0).
    pub edge_density: f32,
    /// Percentage of fully transparent pixels.
    pub transparent_percentage: f32,
    /// Percentage of pixels matching background color.
    pub background_percentage: f32,
    /// Number of distinct connected components (binary threshold).
    pub connected_components: u32,
    /// Dominant colors in the image.
    pub dominant_colors: Vec<[u8; 3]>,
}

/// Color histogram with 16 bins per channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorHistogram {
    pub red: [u32; 16],
    pub green: [u32; 16],
    pub blue: [u32; 16],
    pub alpha: [u32; 16],
}

/// Results of invariant checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantResults {
    pub errors: Vec<InvariantViolation>,
    pub warnings: Vec<InvariantViolation>,
    pub notes: Vec<String>,
    pub overall: OverallStatus,
}

/// Overall status of invariant checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallStatus {
    Pass,
    PassWithWarnings,
    Fail,
}

/// A single invariant violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub category: InvariantCategory,
    pub message: String,
    pub details: Option<String>,
}

/// Categories of invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantCategory {
    Scene,
    Camera,
    Geometry,
    Material,
    Render,
    Stability,
}

impl std::fmt::Display for InvariantCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvariantCategory::Scene => write!(f, "Scene"),
            InvariantCategory::Camera => write!(f, "Camera"),
            InvariantCategory::Geometry => write!(f, "Geometry"),
            InvariantCategory::Material => write!(f, "Material"),
            InvariantCategory::Render => write!(f, "Render"),
            InvariantCategory::Stability => write!(f, "Stability"),
        }
    }
}

impl AuditBundle {
    /// Serialize to pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl InvariantResults {
    /// Create empty results (pass state).
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            notes: Vec::new(),
            overall: OverallStatus::Pass,
        }
    }

    /// Add an error.
    pub fn error(&mut self, category: InvariantCategory, message: impl Into<String>) {
        self.errors.push(InvariantViolation {
            category,
            message: message.into(),
            details: None,
        });
        self.overall = OverallStatus::Fail;
    }

    /// Add an error with details.
    pub fn error_with_details(
        &mut self,
        category: InvariantCategory,
        message: impl Into<String>,
        details: impl Into<String>,
    ) {
        self.errors.push(InvariantViolation {
            category,
            message: message.into(),
            details: Some(details.into()),
        });
        self.overall = OverallStatus::Fail;
    }

    /// Add a warning.
    pub fn warning(&mut self, category: InvariantCategory, message: impl Into<String>) {
        self.warnings.push(InvariantViolation {
            category,
            message: message.into(),
            details: None,
        });
        if self.overall == OverallStatus::Pass {
            self.overall = OverallStatus::PassWithWarnings;
        }
    }

    /// Add a note.
    pub fn note(&mut self, message: impl Into<String>) {
        self.notes.push(message.into());
    }
}

impl Default for InvariantResults {
    fn default() -> Self {
        Self::new()
    }
}
