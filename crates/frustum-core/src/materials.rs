//! Material system for visual appearance.
//!
//! Materials control how geometry looks, never what it is.
//! Geometry references materials by ID.

use serde::{Deserialize, Serialize};

/// A material that can be referenced by geometry primitives.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Material {
    /// Uniform solid color.
    Solid(SolidMaterial),
    /// Scalar-to-color mapping via colormap.
    ScalarMapped(ScalarMappedMaterial),
}

impl Material {
    /// Get the material's ID.
    pub fn id(&self) -> &str {
        match self {
            Material::Solid(m) => &m.id,
            Material::ScalarMapped(m) => &m.id,
        }
    }
}

/// A material with uniform solid color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolidMaterial {
    /// Unique identifier.
    pub id: String,
    /// RGBA color (values in [0, 1]).
    pub color: [f32; 4],
}

impl SolidMaterial {
    /// Create a new solid material with RGB color (alpha = 1.0).
    pub fn new(id: impl Into<String>, rgb: [f32; 3]) -> Self {
        Self {
            id: id.into(),
            color: [rgb[0], rgb[1], rgb[2], 1.0],
        }
    }

    /// Create a new solid material with RGBA color.
    pub fn with_alpha(id: impl Into<String>, rgba: [f32; 4]) -> Self {
        Self {
            id: id.into(),
            color: rgba,
        }
    }
}

/// A material that maps scalar values to colors via a colormap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarMappedMaterial {
    /// Unique identifier.
    pub id: String,
    /// Colormap identifier (e.g., "viridis", "plasma").
    pub colormap: String,
    /// Scalar range [min, max] for normalization.
    pub range: [f32; 2],
    /// Whether to clamp values outside range (default: true).
    #[serde(default = "default_clamp")]
    pub clamp: bool,
    /// Color for NaN or missing values (RGBA).
    #[serde(default = "default_missing_color")]
    pub missing_color: [f32; 4],
}

fn default_clamp() -> bool {
    true
}

fn default_missing_color() -> [f32; 4] {
    [0.5, 0.5, 0.5, 1.0] // Gray
}

impl ScalarMappedMaterial {
    /// Create a new scalar-mapped material.
    pub fn new(id: impl Into<String>, colormap: impl Into<String>, range: [f32; 2]) -> Self {
        Self {
            id: id.into(),
            colormap: colormap.into(),
            range,
            clamp: true,
            missing_color: default_missing_color(),
        }
    }

    /// Set whether to clamp values outside range.
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Set the color for missing/NaN values.
    pub fn with_missing_color(mut self, color: [f32; 4]) -> Self {
        self.missing_color = color;
        self
    }
}

/// Available colormap identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Colormap {
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Cividis,
}

impl Colormap {
    /// Get the colormap name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Colormap::Viridis => "viridis",
            Colormap::Plasma => "plasma",
            Colormap::Inferno => "inferno",
            Colormap::Magma => "magma",
            Colormap::Cividis => "cividis",
        }
    }

    /// Parse a colormap name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "viridis" => Some(Colormap::Viridis),
            "plasma" => Some(Colormap::Plasma),
            "inferno" => Some(Colormap::Inferno),
            "magma" => Some(Colormap::Magma),
            "cividis" => Some(Colormap::Cividis),
            _ => None,
        }
    }

    /// Sample the colormap at a normalized value t in [0, 1].
    /// Returns RGB color.
    pub fn sample(&self, t: f32) -> [f32; 3] {
        let t = t.clamp(0.0, 1.0);
        match self {
            Colormap::Viridis => sample_viridis(t),
            Colormap::Plasma => sample_plasma(t),
            Colormap::Inferno => sample_inferno(t),
            Colormap::Magma => sample_magma(t),
            Colormap::Cividis => sample_cividis(t),
        }
    }
}

// Colormap data - using simplified polynomial approximations
// These are perceptually uniform colormaps

fn sample_viridis(t: f32) -> [f32; 3] {
    // Viridis: dark purple -> blue -> teal -> green -> yellow
    let r = (0.267004 + t * (0.282327 + t * (-0.078908 + t * (2.772570 + t * (-3.263024 + t * 1.228522))))).clamp(0.0, 1.0);
    let g = (0.004874 + t * (1.260580 + t * (-0.221097 + t * (-0.766924 + t * (1.442251 + t * -0.714853))))).clamp(0.0, 1.0);
    let b = (0.329415 + t * (1.701596 + t * (-5.413392 + t * (10.519490 + t * (-8.923144 + t * 2.786102))))).clamp(0.0, 1.0);
    [r, g, b]
}

fn sample_plasma(t: f32) -> [f32; 3] {
    // Plasma: dark blue -> purple -> pink -> orange -> yellow
    let r = (0.050383 + t * (2.023000 + t * (-1.294560 + t * (-0.795670 + t * (1.974810 + t * -0.958000))))).clamp(0.0, 1.0);
    let g = (0.029803 + t * (-0.221780 + t * (1.735400 + t * (-0.719190 + t * (-0.551390 + t * 0.727600))))).clamp(0.0, 1.0);
    let b = (0.527975 + t * (1.573200 + t * (-4.576600 + t * (6.762040 + t * (-4.665700 + t * 1.379000))))).clamp(0.0, 1.0);
    [r, g, b]
}

fn sample_inferno(t: f32) -> [f32; 3] {
    // Inferno: black -> purple -> red -> orange -> yellow
    let r = (0.001462 + t * (1.265980 + t * (0.835940 + t * (-2.371800 + t * (3.010950 + t * -1.735700))))).clamp(0.0, 1.0);
    let g = (0.000466 + t * (-0.055530 + t * (1.827670 + t * (-2.178070 + t * (1.911960 + t * -0.505430))))).clamp(0.0, 1.0);
    let b = (0.013866 + t * (2.066870 + t * (-4.865040 + t * (5.696400 + t * (-3.285300 + t * 0.398620))))).clamp(0.0, 1.0);
    [r, g, b]
}

fn sample_magma(t: f32) -> [f32; 3] {
    // Magma: black -> purple -> pink -> orange -> white
    let r = (0.001462 + t * (1.032690 + t * (0.958610 + t * (-1.681100 + t * (2.341200 + t * -1.654800))))).clamp(0.0, 1.0);
    let g = (0.000466 + t * (-0.267510 + t * (1.912500 + t * (-1.795950 + t * (1.512450 + t * -0.361250))))).clamp(0.0, 1.0);
    let b = (0.013866 + t * (2.377680 + t * (-5.298660 + t * (5.932700 + t * (-3.114900 + t * 0.116820))))).clamp(0.0, 1.0);
    [r, g, b]
}

fn sample_cividis(t: f32) -> [f32; 3] {
    // Cividis: colorblind-friendly, blue -> gray -> yellow
    let r = (-0.046889 + t * (1.573410 + t * (-1.259290 + t * (0.984680 + t * (-0.253910 + t * 0.003180))))).clamp(0.0, 1.0);
    let g = (0.135112 + t * (0.654420 + t * (0.117460 + t * (-0.037870 + t * (0.114390 + t * 0.016420))))).clamp(0.0, 1.0);
    let b = (0.311950 + t * (0.579930 + t * (-1.507500 + t * (1.556530 + t * (-0.735880 + t * 0.114020))))).clamp(0.0, 1.0);
    [r, g, b]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colormap_endpoints() {
        // Viridis should go from dark purple to yellow
        let start = Colormap::Viridis.sample(0.0);
        let end = Colormap::Viridis.sample(1.0);

        // Start should be dark (low luminance)
        assert!(start[0] < 0.4 && start[1] < 0.1);
        // End should be bright yellow-ish
        assert!(end[0] > 0.9 && end[1] > 0.9);
    }

    #[test]
    fn test_colormap_clamping() {
        let below = Colormap::Viridis.sample(-0.5);
        let start = Colormap::Viridis.sample(0.0);
        assert_eq!(below, start);

        let above = Colormap::Viridis.sample(1.5);
        let end = Colormap::Viridis.sample(1.0);
        assert_eq!(above, end);
    }
}
