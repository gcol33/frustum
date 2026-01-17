//! Demonstrate coordinate axes rendering.
//!
//! Run with: cargo run -p frustum-render --example axes

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{Axis, AxisBounds, AxisBundle, Camera, PointCloud, TickSpec};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    // Create a small point cloud to give context
    let mut positions = Vec::new();
    for i in 0..20 {
        let t = i as f32 / 20.0;
        let x = t * 2.0 - 1.0;
        let y = (t * 4.0 * std::f32::consts::PI).sin() * 0.5;
        let z = (t * 4.0 * std::f32::consts::PI).cos() * 0.5;
        positions.extend_from_slice(&[x, y, z]);
    }
    let point_cloud = PointCloud::new(positions, 6.0);

    // Create coordinate axes
    let axes = AxisBundle::new(
        "main_axes",
        AxisBounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .with_axes(vec![Axis::X, Axis::Y, Axis::Z])
    .with_ticks(TickSpec::Auto { count: 4 })
    .with_line_width(1.0);

    // Create camera
    let camera = Camera::perspective([2.5, 2.0, 2.5], [0.0, 0.0, 0.0], 45.0);

    // Create scene
    let bounds = Bounds {
        min: [-1.2, -1.2, -1.2],
        max: [1.2, 1.2, 1.2],
    };
    let scene = Scene::new(camera, bounds)
        .add_point_cloud(point_cloud)
        .add_axes(axes);

    // Render
    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.1, 0.1, 0.15, 1.0],
    };

    println!("Rendering scene with coordinate axes...");
    let png_data = render_to_png(&scene, &config).expect("Failed to render");

    // Write PNG
    fs::write("axes.png", &png_data).expect("Failed to write PNG");
    println!("Wrote axes.png ({} bytes)", png_data.len());
}
