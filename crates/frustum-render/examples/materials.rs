//! Demonstrate material system with solid colors and colormaps.
//!
//! Run with: cargo run -p frustum-render --example materials

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{
    Camera, Material, Mesh, PointCloud, Polyline,
    ScalarMappedMaterial, SolidMaterial,
};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    // Create materials
    let red_material = Material::Solid(SolidMaterial::new("red", [0.9, 0.2, 0.2]));
    let blue_material = Material::Solid(SolidMaterial::new("blue", [0.2, 0.4, 0.9]));
    let viridis_material = Material::ScalarMapped(
        ScalarMappedMaterial::new("viridis_map", "viridis", [0.0, 1.0])
    );
    let plasma_material = Material::ScalarMapped(
        ScalarMappedMaterial::new("plasma_map", "plasma", [-1.0, 1.0])
    );

    // Create a simple cube mesh with red material
    let cube_positions = vec![
        // Front face
        -0.3, -0.3,  0.3,   0.3, -0.3,  0.3,   0.3,  0.3,  0.3,  -0.3,  0.3,  0.3,
        // Back face
        -0.3, -0.3, -0.3,  -0.3,  0.3, -0.3,   0.3,  0.3, -0.3,   0.3, -0.3, -0.3,
    ];
    let cube_indices = vec![
        0, 1, 2, 0, 2, 3,  // Front
        4, 5, 6, 4, 6, 7,  // Back
        0, 3, 5, 0, 5, 4,  // Left
        1, 7, 6, 1, 6, 2,  // Right
        3, 2, 6, 3, 6, 5,  // Top
        0, 4, 7, 0, 7, 1,  // Bottom
    ];
    let cube = Mesh::new(cube_positions, cube_indices)
        .with_material("red");

    // Create point cloud with scalar-mapped colors (viridis)
    let mut point_positions = Vec::new();
    let mut point_scalars = Vec::new();
    for i in 0..50 {
        let t = i as f32 / 50.0;
        let angle = t * 4.0 * std::f32::consts::PI;
        let radius = 0.6 + t * 0.2;
        let x = radius * angle.cos();
        let y = t - 0.5;
        let z = radius * angle.sin();
        point_positions.extend_from_slice(&[x, y, z]);
        point_scalars.push(t); // Scalar from 0 to 1
    }
    let points = PointCloud::new(point_positions, 8.0)
        .with_scalars(point_scalars)
        .with_material("viridis_map");

    // Create a polyline with blue material
    let mut line_positions = Vec::new();
    for i in 0..30 {
        let t = i as f32 / 30.0;
        let angle = t * 2.0 * std::f32::consts::PI;
        let x = 0.8 * angle.cos();
        let y = -0.6;
        let z = 0.8 * angle.sin();
        line_positions.extend_from_slice(&[x, y, z]);
    }
    // Close the loop
    line_positions.extend_from_slice(&[line_positions[0], line_positions[1], line_positions[2]]);
    let polyline = Polyline::new(line_positions, 2.0)
        .with_material("blue");

    // Create another point cloud with plasma colormap
    let mut plasma_positions = Vec::new();
    let mut plasma_scalars = Vec::new();
    for i in 0..30 {
        let t = i as f32 / 30.0;
        let angle = t * 2.0 * std::f32::consts::PI;
        let x = 0.5 * angle.cos();
        let y = 0.6;
        let z = 0.5 * angle.sin();
        plasma_positions.extend_from_slice(&[x, y, z]);
        plasma_scalars.push((angle).sin()); // Scalar from -1 to 1
    }
    let plasma_points = PointCloud::new(plasma_positions, 10.0)
        .with_scalars(plasma_scalars)
        .with_material("plasma_map");

    // Create camera
    let camera = Camera::perspective([2.0, 1.5, 2.0], [0.0, 0.0, 0.0], 45.0);

    // Create scene
    let bounds = Bounds {
        min: [-1.5, -1.5, -1.5],
        max: [1.5, 1.5, 1.5],
    };
    let scene = Scene::new(camera, bounds)
        .add_material(red_material)
        .add_material(blue_material)
        .add_material(viridis_material)
        .add_material(plasma_material)
        .add_mesh(cube)
        .add_point_cloud(points)
        .add_point_cloud(plasma_points)
        .add_polyline(polyline);

    // Render
    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.05, 0.05, 0.1, 1.0],
    };

    println!("Rendering scene with materials...");
    let png_data = render_to_png(&scene, &config).expect("Failed to render");

    // Write PNG
    fs::write("materials.png", &png_data).expect("Failed to write PNG");
    println!("Wrote materials.png ({} bytes)", png_data.len());
    println!("Materials: red (solid), blue (solid), viridis (scalar-mapped), plasma (scalar-mapped)");
}
