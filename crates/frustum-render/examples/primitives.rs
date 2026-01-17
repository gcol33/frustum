//! Demonstrate point cloud and polyline rendering.
//!
//! Run with: cargo run -p frustum-render --example primitives

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{Camera, PointCloud, Polyline};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    // Create a point cloud: random points in a cube
    let mut positions = Vec::new();
    for i in 0..100 {
        let t = i as f32 / 100.0;
        // Spiral pattern
        let angle = t * 6.0 * std::f32::consts::PI;
        let radius = 0.3 + t * 0.3;
        let x = radius * angle.cos();
        let y = t - 0.5;
        let z = radius * angle.sin();
        positions.extend_from_slice(&[x, y, z]);
    }
    let point_cloud = PointCloud::new(positions.clone(), 4.0);

    // Create a polyline: helix
    let mut line_positions = Vec::new();
    for i in 0..50 {
        let t = i as f32 / 50.0;
        let angle = t * 4.0 * std::f32::consts::PI;
        let x = 0.4 * angle.cos();
        let y = t - 0.5;
        let z = 0.4 * angle.sin();
        line_positions.extend_from_slice(&[x, y, z]);
    }
    let polyline = Polyline::new(line_positions, 1.0);

    // Create camera
    let camera = Camera::perspective([1.5, 1.0, 1.5], [0.0, 0.0, 0.0], 45.0);

    // Create scene with point cloud and polyline
    let bounds = Bounds {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };
    let scene = Scene::new(camera, bounds)
        .add_point_cloud(point_cloud)
        .add_polyline(polyline);

    // Render
    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.05, 0.05, 0.1, 1.0],
    };

    println!("Rendering point cloud and polyline...");
    let png_data = render_to_png(&scene, &config).expect("Failed to render");

    // Write PNG
    fs::write("primitives.png", &png_data).expect("Failed to write PNG");
    println!("Wrote primitives.png ({} bytes)", png_data.len());
    println!("Scene has {} points and 50 line vertices", positions.len() / 3);
}
