//! Material system for visual appearance.
//!
//! Materials control how geometry looks, never what it is.
//! Geometry references materials by ID.

use serde::{Deserialize, Serialize};
use crate::colormap_tables;

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

/// Number of entries in each colormap table.
pub const COLORMAP_TABLE_LEN: usize = colormap_tables::TABLE_LEN;

impl Colormap {
    /// All colormaps, in the order of [`Colormap::index`].
    pub const ALL: [Colormap; 5] = [
        Colormap::Viridis,
        Colormap::Plasma,
        Colormap::Inferno,
        Colormap::Magma,
        Colormap::Cividis,
    ];

    /// Position of this colormap in [`Colormap::ALL`].
    pub fn index(&self) -> usize {
        match self {
            Colormap::Viridis => 0,
            Colormap::Plasma => 1,
            Colormap::Inferno => 2,
            Colormap::Magma => 3,
            Colormap::Cividis => 4,
        }
    }

    /// The colormap's sRGB table, sampled at t = i / (len - 1).
    pub fn table(&self) -> &'static [[f32; 3]; COLORMAP_TABLE_LEN] {
        match self {
            Colormap::Viridis => &colormap_tables::VIRIDIS,
            Colormap::Plasma => &colormap_tables::PLASMA,
            Colormap::Inferno => &colormap_tables::INFERNO,
            Colormap::Magma => &colormap_tables::MAGMA,
            Colormap::Cividis => &colormap_tables::CIVIDIS,
        }
    }

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
    /// Returns the sRGB colour as matplotlib defines it.
    pub fn sample(&self, t: f32) -> [f32; 3] {
        let t = t.clamp(0.0, 1.0);
        sample_table(self.table(), t)
    }
}

/// Look up a colormap table at t in [0, 1] with linear interpolation
/// between the two nearest entries.
fn sample_table(table: &[[f32; 3]; colormap_tables::TABLE_LEN], t: f32) -> [f32; 3] {
    let x = t * (colormap_tables::TABLE_LEN - 1) as f32;
    let i = (x.floor() as usize).min(colormap_tables::TABLE_LEN - 2);
    let f = x - i as f32;
    let a = table[i];
    let b = table[i + 1];
    [
        a[0] + (b[0] - a[0]) * f,
        a[1] + (b[1] - a[1]) * f,
        a[2] + (b[2] - a[2]) * f,
    ]
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
