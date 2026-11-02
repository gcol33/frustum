//! Frustum Core
//!
//! Core scene model and geometry primitives for the Frustum rendering framework.

pub mod camera;
mod colormap_tables;
pub mod geometry;
pub mod lighting;
pub mod marching_cubes;
pub mod materials;
pub mod scene;

pub use camera::{Camera, Projection};
pub use geometry::{Axis, AxisBounds, AxisBundle, Label, LabelSpec, Mesh, PointCloud, Polyline, TickSpec};
pub use lighting::Light;
pub use marching_cubes::{marching_cubes, marching_cubes_multi, IsoSurface, Volume};
pub use materials::{Colormap, Material, ScalarMappedMaterial, SolidMaterial, COLORMAP_TABLE_LEN};
pub use scene::Scene;
