//! Invariant checking for rendered figures.
//!
//! This module implements the actual invariant checks that validate
//! scene correctness, camera setup, geometry validity, and render output.

use crate::audit::{
    AuditBundle, GeometryProbes, ImageMetrics, InvariantCategory, InvariantResults,
    RenderMetadata,
};
use frustum_core::Scene;

/// Check all invariants for a rendered scene.
pub fn check_all_invariants(
    scene: &Scene,
    metadata: &RenderMetadata,
    geometry: &GeometryProbes,
    image: &ImageMetrics,
) -> InvariantResults {
    let mut results = InvariantResults::new();

    check_scene_invariants(scene, metadata, &mut results);
    check_camera_invariants(scene, geometry, &mut results);
    check_geometry_invariants(geometry, metadata, &mut results);
    check_render_invariants(image, metadata, &mut results);

    results
}

/// Scene invariants: geometry within bounds, valid materials, no forbidden combinations.
fn check_scene_invariants(
    scene: &Scene,
    metadata: &RenderMetadata,
    results: &mut InvariantResults,
) {
    use frustum_core::scene::SceneElement;

    // Check that scene has geometry
    if scene.elements.is_empty() {
        results.warning(
            InvariantCategory::Scene,
            "Scene contains no geometry elements",
        );
    }

    // Check geometry within world bounds
    let bounds = &scene.bounds;
    for (i, element) in scene.elements.iter().enumerate() {
        match element {
            SceneElement::Mesh(mesh) => {
                for chunk in mesh.positions.chunks(3) {
                    if chunk.len() == 3 {
                        let x = chunk[0];
                        let y = chunk[1];
                        let z = chunk[2];

                        if x < bounds.min[0]
                            || x > bounds.max[0]
                            || y < bounds.min[1]
                            || y > bounds.max[1]
                            || z < bounds.min[2]
                            || z > bounds.max[2]
                        {
                            results.warning(
                                InvariantCategory::Scene,
                                format!(
                                    "Mesh {} has vertex ({:.2}, {:.2}, {:.2}) outside world bounds",
                                    i, x, y, z
                                ),
                            );
                            break; // Only report once per mesh
                        }

                        // Check for NaN/Inf
                        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                            results.error(
                                InvariantCategory::Scene,
                                format!("Mesh {} contains NaN or Inf vertex positions", i),
                            );
                            break;
                        }
                    }
                }
            }
            SceneElement::PointCloud(pc) => {
                for chunk in pc.positions.chunks(3) {
                    if chunk.len() == 3 && (!chunk[0].is_finite() || !chunk[1].is_finite() || !chunk[2].is_finite()) {
                        results.error(
                            InvariantCategory::Scene,
                            format!("PointCloud {} contains NaN or Inf positions", i),
                        );
                        break;
                    }
                }
            }
            SceneElement::Polyline(line) => {
                for chunk in line.positions.chunks(3) {
                    if chunk.len() == 3 && (!chunk[0].is_finite() || !chunk[1].is_finite() || !chunk[2].is_finite()) {
                        results.error(
                            InvariantCategory::Scene,
                            format!("Polyline {} contains NaN or Inf positions", i),
                        );
                        break;
                    }
                }
            }
            SceneElement::Axes(axes) => {
                // Validate axis bounds
                let ab = &axes.bounds;
                if ab.min[0] > ab.max[0] || ab.min[1] > ab.max[1] || ab.min[2] > ab.max[2] {
                    results.error(
                        InvariantCategory::Scene,
                        format!("Axes {} has degenerate bounds", i),
                    );
                }
                // Check if axes bounds exceed scene bounds
                if ab.min[0] < bounds.min[0] || ab.min[1] < bounds.min[1] || ab.min[2] < bounds.min[2]
                    || ab.max[0] > bounds.max[0] || ab.max[1] > bounds.max[1] || ab.max[2] > bounds.max[2]
                {
                    results.warning(
                        InvariantCategory::Scene,
                        format!("Axes {} bounds exceed scene world bounds", i),
                    );
                }
            }
        }
    }

    // Note primitive counts
    results.note(format!(
        "Scene contains {} meshes ({} triangles), {} point clouds, {} polylines",
        metadata.primitive_counts.meshes,
        metadata.primitive_counts.total_triangles,
        metadata.primitive_counts.point_clouds,
        metadata.primitive_counts.polylines
    ));
}

/// Camera invariants: geometry visible, not everything clipped, no NaN projections.
fn check_camera_invariants(
    scene: &Scene,
    geometry: &GeometryProbes,
    results: &mut InvariantResults,
) {
    // Check camera position != target
    let cam = &scene.camera;
    let dist = ((cam.position[0] - cam.target[0]).powi(2)
        + (cam.position[1] - cam.target[1]).powi(2)
        + (cam.position[2] - cam.target[2]).powi(2))
    .sqrt();

    if dist < 1e-6 {
        results.error(
            InvariantCategory::Camera,
            "Camera position equals target (degenerate view)",
        );
    }

    // Check near < far
    if cam.near >= cam.far {
        results.error(
            InvariantCategory::Camera,
            format!("Camera near ({}) >= far ({})", cam.near, cam.far),
        );
    }

    // Check near > 0
    if cam.near <= 0.0 {
        results.error(
            InvariantCategory::Camera,
            format!("Camera near plane ({}) must be positive", cam.near),
        );
    }

    // Check geometry visibility
    if !geometry.geometry_visible {
        results.warning(
            InvariantCategory::Camera,
            "No geometry visible in view frustum (everything clipped or outside view)",
        );
    }

    // Check for NaN in projections
    if geometry.has_invalid_values {
        results.error(
            InvariantCategory::Camera,
            "NaN or Inf detected in projected coordinates",
        );
    }

    // Check depth stats
    if geometry.depth_stats.far_plane_percentage > 99.0 {
        results.warning(
            InvariantCategory::Camera,
            format!(
                "{:.1}% of pixels at far plane (scene may be empty or camera misaligned)",
                geometry.depth_stats.far_plane_percentage
            ),
        );
    }
}

/// Geometry invariants: non-zero primitives, degenerate handling, topology.
fn check_geometry_invariants(
    geometry: &GeometryProbes,
    metadata: &RenderMetadata,
    results: &mut InvariantResults,
) {
    // Check for degenerate primitives
    if geometry.degenerate_count > 0 {
        results.warning(
            InvariantCategory::Geometry,
            format!(
                "{} degenerate primitives detected (zero-area triangles, etc.)",
                geometry.degenerate_count
            ),
        );
    }

    // Check clipped primitives
    if geometry.clipped_count > 0 {
        let total = metadata.primitive_counts.total_triangles;
        let pct = if total > 0 {
            (geometry.clipped_count as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        if pct > 50.0 {
            results.warning(
                InvariantCategory::Geometry,
                format!(
                    "{} primitives clipped ({:.1}% of total) - consider adjusting camera planes",
                    geometry.clipped_count, pct
                ),
            );
        } else {
            results.note(format!(
                "{} primitives clipped by near/far planes",
                geometry.clipped_count
            ));
        }
    }

    // Note back-face culling
    if geometry.backface_count > 0 {
        results.note(format!(
            "{} back-facing triangles culled",
            geometry.backface_count
        ));
    }
}

/// Render invariants: background applied, resolution correct, alpha as expected.
fn check_render_invariants(
    image: &ImageMetrics,
    metadata: &RenderMetadata,
    results: &mut InvariantResults,
) {
    // Check resolution
    if metadata.resolution[0] == 0 || metadata.resolution[1] == 0 {
        results.error(
            InvariantCategory::Render,
            format!(
                "Invalid resolution: {}x{}",
                metadata.resolution[0], metadata.resolution[1]
            ),
        );
    }

    // Check for mostly transparent image (might indicate render failure)
    if image.transparent_percentage > 99.0 {
        results.warning(
            InvariantCategory::Render,
            format!(
                "{:.1}% transparent pixels - render may have failed",
                image.transparent_percentage
            ),
        );
    }

    // Check for mostly background (nothing rendered)
    if image.background_percentage > 99.0 {
        results.warning(
            InvariantCategory::Render,
            format!(
                "{:.1}% of image is background color - scene may be empty or not visible",
                image.background_percentage
            ),
        );
    }

    // Check edge density (very low might indicate solid color / render failure)
    if image.edge_density < 0.001 && image.background_percentage < 100.0 {
        results.note(
            "Very low edge density - image may be mostly flat colors".to_string(),
        );
    }

    // Note connected components
    if image.connected_components > 0 {
        results.note(format!(
            "{} distinct regions detected in rendered image",
            image.connected_components
        ));
    }
}

/// Compare two audit bundles for regression testing.
pub fn compare_for_regression(
    baseline: &AuditBundle,
    current: &AuditBundle,
    tolerance: RegressionTolerance,
) -> RegressionResult {
    let mut result = RegressionResult {
        matches: true,
        differences: Vec::new(),
        notes: Vec::new(),
    };

    // Compare primitive counts (must match exactly)
    if baseline.metadata.primitive_counts.total_triangles
        != current.metadata.primitive_counts.total_triangles
    {
        result.matches = false;
        result.differences.push(format!(
            "Triangle count changed: {} -> {}",
            baseline.metadata.primitive_counts.total_triangles,
            current.metadata.primitive_counts.total_triangles
        ));
    }

    // Compare depth stats within tolerance
    let depth_diff =
        (baseline.geometry.depth_stats.mean - current.geometry.depth_stats.mean).abs();
    if depth_diff > tolerance.depth_tolerance {
        result.matches = false;
        result.differences.push(format!(
            "Mean depth changed beyond tolerance: {:.4} -> {:.4} (diff: {:.4}, tolerance: {:.4})",
            baseline.geometry.depth_stats.mean,
            current.geometry.depth_stats.mean,
            depth_diff,
            tolerance.depth_tolerance
        ));
    }

    // Compare histogram (color distribution)
    let hist_diff = histogram_difference(
        &baseline.image_metrics.histogram,
        &current.image_metrics.histogram,
    );
    if hist_diff > tolerance.histogram_tolerance {
        result.matches = false;
        result.differences.push(format!(
            "Color histogram drift: {:.2}% (tolerance: {:.2}%)",
            hist_diff * 100.0,
            tolerance.histogram_tolerance * 100.0
        ));
    }

    // Compare edge density
    let edge_diff =
        (baseline.image_metrics.edge_density - current.image_metrics.edge_density).abs();
    if edge_diff > tolerance.edge_density_tolerance {
        result.differences.push(format!(
            "Edge density changed: {:.4} -> {:.4}",
            baseline.image_metrics.edge_density, current.image_metrics.edge_density
        ));
        // Edge density changes are warnings, not failures
    }

    // Compare background percentage
    let bg_diff = (baseline.image_metrics.background_percentage
        - current.image_metrics.background_percentage)
        .abs();
    if bg_diff > tolerance.background_tolerance {
        result.matches = false;
        result.differences.push(format!(
            "Background percentage changed: {:.1}% -> {:.1}%",
            baseline.image_metrics.background_percentage,
            current.image_metrics.background_percentage
        ));
    }

    // Note backend differences
    if baseline.metadata.backend != current.metadata.backend {
        result.notes.push(format!(
            "Different GPU backend: {} vs {}",
            baseline.metadata.backend, current.metadata.backend
        ));
    }

    result
}

/// Tolerance settings for regression comparison.
#[derive(Debug, Clone)]
pub struct RegressionTolerance {
    pub depth_tolerance: f32,
    pub histogram_tolerance: f32,
    pub edge_density_tolerance: f32,
    pub background_tolerance: f32,
}

impl Default for RegressionTolerance {
    fn default() -> Self {
        Self {
            depth_tolerance: 0.01,
            histogram_tolerance: 0.05,
            edge_density_tolerance: 0.1,
            background_tolerance: 5.0,
        }
    }
}

/// Result of regression comparison.
#[derive(Debug, Clone)]
pub struct RegressionResult {
    pub matches: bool,
    pub differences: Vec<String>,
    pub notes: Vec<String>,
}

/// Calculate normalized difference between two histograms.
fn histogram_difference(
    a: &crate::audit::ColorHistogram,
    b: &crate::audit::ColorHistogram,
) -> f32 {
    let mut total_diff = 0u64;
    let mut total_count = 0u64;

    for i in 0..16 {
        total_diff += (a.red[i] as i64 - b.red[i] as i64).unsigned_abs();
        total_diff += (a.green[i] as i64 - b.green[i] as i64).unsigned_abs();
        total_diff += (a.blue[i] as i64 - b.blue[i] as i64).unsigned_abs();
        total_count += a.red[i] as u64 + a.green[i] as u64 + a.blue[i] as u64;
    }

    if total_count == 0 {
        return 0.0;
    }

    total_diff as f32 / total_count as f32
}
