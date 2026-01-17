//! Lighting model for Frustum scenes.
//!
//! Frustum uses a minimal, deterministic lighting model:
//! - Single directional light with Lambertian shading
//! - Only meshes with normals receive shading
//! - Points, lines, and axes render unlit (flat color)
//! - No light specified = flat colors (no implicit headlight)

use serde::{Deserialize, Serialize};

/// A directional light for Lambertian shading.
///
/// Light direction points toward the light source (not the direction light travels).
/// Only affects meshes with normals; other primitives render unlit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Light {
    /// Direction toward the light source (normalized, world space).
    pub direction: [f32; 3],
    /// Light intensity (>= 0). Multiplies the diffuse term.
    pub intensity: f32,
    /// Whether lighting is applied. If false, meshes render flat.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Light {
    /// Create a new directional light.
    ///
    /// Direction is automatically normalized.
    ///
    /// # Panics
    /// Panics if direction is zero-length or intensity is negative.
    pub fn new(direction: [f32; 3], intensity: f32) -> Self {
        let len = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
        .sqrt();

        assert!(len > 1e-6, "Light direction must be non-zero");
        assert!(intensity >= 0.0, "Light intensity must be non-negative");
        assert!(intensity.is_finite(), "Light intensity must be finite");

        Self {
            direction: [direction[0] / len, direction[1] / len, direction[2] / len],
            intensity,
            enabled: true,
        }
    }

    /// Create a light with custom enabled state.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    // =========================================================================
    // Lighting Presets (from 009-extensions.md)
    // =========================================================================

    /// Scientific flat lighting: overhead light, moderate intensity.
    ///
    /// Good for data visualization where shape is important but not dramatic.
    pub fn scientific_flat() -> Self {
        Self::new([0.0, 1.0, 0.3], 0.8)
    }

    /// Studio soft lighting: front-top-right, balanced intensity.
    ///
    /// Classic 3D rendering setup, good for general purpose visualization.
    pub fn studio_soft() -> Self {
        Self::new([0.5, 0.7, 0.5], 1.0)
    }

    /// Rim highlight lighting: back-top light for edge emphasis.
    ///
    /// Creates bright edges on geometry, good for showing silhouettes.
    pub fn rim_highlight() -> Self {
        Self::new([-0.3, 0.5, -0.8], 1.2)
    }

    /// Depth emphasis lighting: steep top-down angle.
    ///
    /// Emphasizes depth variations and surface detail.
    pub fn depth_emphasis() -> Self {
        Self::new([0.1, 0.95, 0.1], 1.0)
    }

    /// Side lighting: strong lateral illumination.
    ///
    /// Emphasizes surface topology and small features.
    pub fn side_light() -> Self {
        Self::new([1.0, 0.3, 0.2], 1.0)
    }

    /// Three-quarter view lighting: classic 45-degree setup.
    ///
    /// Balanced illumination from front-top-left, most versatile preset.
    pub fn three_quarter() -> Self {
        Self::new([0.577, 0.577, 0.577], 1.0) // Normalized (1,1,1)
    }

    /// Validate the light configuration.
    ///
    /// Returns an error message if invalid, None if valid.
    pub fn validate(&self) -> Option<String> {
        // Check direction for NaN/Inf
        for (i, &v) in self.direction.iter().enumerate() {
            if !v.is_finite() {
                return Some(format!("Light direction[{}] is not finite: {}", i, v));
            }
        }

        // Check direction is normalized
        let len = (self.direction[0] * self.direction[0]
            + self.direction[1] * self.direction[1]
            + self.direction[2] * self.direction[2])
        .sqrt();

        if len < 0.99 || len > 1.01 {
            return Some(format!(
                "Light direction is not normalized: length = {}",
                len
            ));
        }

        // Check intensity
        if !self.intensity.is_finite() {
            return Some(format!(
                "Light intensity is not finite: {}",
                self.intensity
            ));
        }

        if self.intensity < 0.0 {
            return Some(format!(
                "Light intensity must be non-negative: {}",
                self.intensity
            ));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_normalization() {
        let light = Light::new([1.0, 1.0, 1.0], 1.0);
        let len = (light.direction[0].powi(2)
            + light.direction[1].powi(2)
            + light.direction[2].powi(2))
        .sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_light_validation() {
        let valid = Light::new([0.0, 0.0, 1.0], 1.0);
        assert!(valid.validate().is_none());

        let invalid_intensity = Light {
            direction: [0.0, 0.0, 1.0],
            intensity: -1.0,
            enabled: true,
        };
        assert!(invalid_intensity.validate().is_some());
    }

    #[test]
    #[should_panic(expected = "non-zero")]
    fn test_zero_direction_panics() {
        Light::new([0.0, 0.0, 0.0], 1.0);
    }

    #[test]
    #[should_panic(expected = "non-negative")]
    fn test_negative_intensity_panics() {
        Light::new([0.0, 0.0, 1.0], -1.0);
    }
}
