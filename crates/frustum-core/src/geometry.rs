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
    /// Material ID reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_id: Option<String>,
}

impl PointCloud {
    /// Create a new point cloud from positions.
    pub fn new(positions: Vec<f32>, point_size: f32) -> Self {
        Self {
            positions,
            scalars: None,
            point_size,
            material_id: None,
        }
    }

    /// Set scalar values for colormap mapping.
    pub fn with_scalars(mut self, scalars: Vec<f32>) -> Self {
        self.scalars = Some(scalars);
        self
    }

    /// Set material ID.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.material_id = Some(material_id.into());
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
    /// Material ID reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_id: Option<String>,
}

impl Polyline {
    /// Create a new polyline from positions.
    pub fn new(positions: Vec<f32>, line_width: f32) -> Self {
        Self {
            positions,
            line_width,
            material_id: None,
        }
    }

    /// Set material ID.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.material_id = Some(material_id.into());
        self
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
    /// Material ID reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material_id: Option<String>,
}

/// Tick generation mode for axes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum TickSpec {
    /// Fixed tick positions in world coordinates.
    Fixed { values: Vec<f32> },
    /// Automatic tick generation with approximate count.
    Auto { count: u32 },
    /// No ticks.
    None,
}

impl Default for TickSpec {
    fn default() -> Self {
        TickSpec::Auto { count: 5 }
    }
}

/// Label specification for axis ticks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSpec {
    /// Whether to generate label placeholders.
    pub show: bool,
    /// World-space offset from tick position.
    pub offset: [f32; 3],
    /// Format string for numeric labels (e.g., "%.2f").
    pub format: Option<String>,
}

impl Default for LabelSpec {
    fn default() -> Self {
        Self {
            show: true,
            offset: [0.0, 0.0, 0.0], // Additional user offset (axis-specific offset computed automatically)
            format: None,
        }
    }
}

/// A label placeholder with position and text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// World-space position.
    pub position: [f32; 3],
    /// Label text.
    pub text: String,
}

/// Coordinate axes as explicit geometry.
///
/// Axes expand into Lines primitives for rendering.
/// No special-casing in the renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisBundle {
    /// Unique identifier.
    pub id: String,
    /// Axis-aligned bounding box for the axes.
    pub bounds: AxisBounds,
    /// Which axes to render (subset of x, y, z).
    pub axes: Vec<Axis>,
    /// Line width for axis lines and ticks.
    pub line_width: f32,
    /// Tick specification.
    #[serde(default)]
    pub ticks: TickSpec,
    /// Label specification.
    #[serde(default)]
    pub labels: LabelSpec,
}

/// Bounds for axis bundle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Which axis (X, Y, or Z).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

impl AxisBundle {
    /// Create a new axis bundle with default settings.
    pub fn new(id: impl Into<String>, bounds: AxisBounds) -> Self {
        Self {
            id: id.into(),
            bounds,
            axes: vec![Axis::X, Axis::Y, Axis::Z],
            line_width: 1.0,
            ticks: TickSpec::default(),
            labels: LabelSpec::default(),
        }
    }

    /// Set which axes to display.
    pub fn with_axes(mut self, axes: Vec<Axis>) -> Self {
        self.axes = axes;
        self
    }

    /// Set line width.
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Set tick specification.
    pub fn with_ticks(mut self, ticks: TickSpec) -> Self {
        self.ticks = ticks;
        self
    }

    /// Expand axes into polylines for rendering.
    ///
    /// Returns a list of polylines (axis lines + tick marks) and labels.
    pub fn expand(&self) -> (Vec<Polyline>, Vec<Label>) {
        let mut polylines = Vec::new();
        let mut labels = Vec::new();

        let [xmin, ymin, zmin] = self.bounds.min;
        let [xmax, ymax, zmax] = self.bounds.max;

        // Generate tick values
        let tick_values = |min: f32, max: f32| -> Vec<f32> {
            match &self.ticks {
                TickSpec::Fixed { values } => values
                    .iter()
                    .filter(|&&v| v >= min && v <= max)
                    .copied()
                    .collect(),
                TickSpec::Auto { count } => {
                    if *count == 0 {
                        return vec![];
                    }
                    let step = (max - min) / (*count as f32);
                    (0..=*count).map(|i| min + i as f32 * step).collect()
                }
                TickSpec::None => vec![],
            }
        };

        let tick_size = (xmax - xmin).min(ymax - ymin).min(zmax - zmin) * 0.02;

        for axis in &self.axes {
            match axis {
                Axis::X => {
                    // Main X axis line at y=ymin, z=zmin
                    polylines.push(Polyline::new(
                        vec![xmin, ymin, zmin, xmax, ymin, zmin],
                        self.line_width,
                    ));

                    // Ticks along X
                    let label_offset_y = -tick_size * 1.5; // Push labels below tick marks
                    for x in tick_values(xmin, xmax) {
                        // Tick mark perpendicular to X (in Y direction)
                        polylines.push(Polyline::new(
                            vec![x, ymin, zmin, x, ymin - tick_size, zmin],
                            self.line_width,
                        ));

                        if self.labels.show {
                            labels.push(Label {
                                position: [
                                    x + self.labels.offset[0],
                                    ymin - tick_size + label_offset_y + self.labels.offset[1],
                                    zmin + self.labels.offset[2],
                                ],
                                text: format_tick_value(x, &self.labels.format),
                            });
                        }
                    }
                }
                Axis::Y => {
                    // Main Y axis line at x=xmin, z=zmin
                    polylines.push(Polyline::new(
                        vec![xmin, ymin, zmin, xmin, ymax, zmin],
                        self.line_width,
                    ));

                    // Ticks along Y
                    let label_offset_x = -tick_size * 1.5; // Push labels left of tick marks
                    for y in tick_values(ymin, ymax) {
                        // Tick mark perpendicular to Y (in X direction)
                        polylines.push(Polyline::new(
                            vec![xmin, y, zmin, xmin - tick_size, y, zmin],
                            self.line_width,
                        ));

                        if self.labels.show {
                            labels.push(Label {
                                position: [
                                    xmin - tick_size + label_offset_x + self.labels.offset[0],
                                    y + self.labels.offset[1],
                                    zmin + self.labels.offset[2],
                                ],
                                text: format_tick_value(y, &self.labels.format),
                            });
                        }
                    }
                }
                Axis::Z => {
                    // Main Z axis line at x=xmin, y=ymin
                    polylines.push(Polyline::new(
                        vec![xmin, ymin, zmin, xmin, ymin, zmax],
                        self.line_width,
                    ));

                    // Ticks along Z
                    let label_offset_x = -tick_size * 1.5; // Push labels left of tick marks
                    for z in tick_values(zmin, zmax) {
                        // Tick mark perpendicular to Z (in X direction)
                        polylines.push(Polyline::new(
                            vec![xmin, ymin, z, xmin - tick_size, ymin, z],
                            self.line_width,
                        ));

                        if self.labels.show {
                            labels.push(Label {
                                position: [
                                    xmin - tick_size + label_offset_x + self.labels.offset[0],
                                    ymin + self.labels.offset[1],
                                    z + self.labels.offset[2],
                                ],
                                text: format_tick_value(z, &self.labels.format),
                            });
                        }
                    }
                }
            }
        }

        (polylines, labels)
    }
}

fn format_tick_value(value: f32, format: &Option<String>) -> String {
    match format {
        Some(fmt) if fmt.contains('%') => {
            // Simple printf-style formatting (just %.Nf for now)
            if let Some(precision) = fmt.strip_prefix("%.").and_then(|s| s.strip_suffix('f')) {
                if let Ok(p) = precision.parse::<usize>() {
                    return format!("{:.prec$}", value, prec = p);
                }
            }
            format!("{}", value)
        }
        _ => {
            // Default: smart formatting for scientific figures
            let abs_val = value.abs();

            if abs_val == 0.0 {
                // Zero is just zero
                "0".to_string()
            } else if abs_val < 0.001 || abs_val >= 10000.0 {
                // Very small or very large: use scientific notation
                // Format and clean up trailing zeros in mantissa
                let s = format!("{:.1e}", value);
                s.replace("e0", "").replace("e-0", "e-").replace("e+", "e")
            } else if abs_val < 0.1 {
                // Small values: show 3 decimal places
                format_trim_zeros(value, 3)
            } else if abs_val < 10.0 {
                // Normal range: show 2 decimal places
                format_trim_zeros(value, 2)
            } else {
                // Larger values: show 1 decimal place
                format_trim_zeros(value, 1)
            }
        }
    }
}

/// Format a number with given precision, trimming unnecessary trailing zeros.
fn format_trim_zeros(value: f32, precision: usize) -> String {
    let s = format!("{:.prec$}", value, prec = precision);
    // Trim trailing zeros after decimal point, but keep at least one digit
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{}0", trimmed)
        } else {
            trimmed.to_string()
        }
    } else {
        s
    }
}

impl Mesh {
    /// Create a new mesh from positions and indices.
    pub fn new(positions: Vec<f32>, indices: Vec<u32>) -> Self {
        Self {
            positions,
            indices,
            normals: None,
            scalars: None,
            material_id: None,
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

    /// Set material ID.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.material_id = Some(material_id.into());
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
