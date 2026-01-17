//! Demonstrate marching cubes isosurface extraction.
//!
//! Run with: cargo run -p frustum-render --example marching_cubes

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{marching_cubes, Camera, Material, SolidMaterial, Volume};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    // Create a 3D scalar field: sphere signed distance function
    let resolution = 32;
    let size = 2.0;
    let spacing = size / (resolution - 1) as f32;
    let origin = [-size / 2.0, -size / 2.0, -size / 2.0];

    let mut values = Vec::with_capacity(resolution * resolution * resolution);
    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                let px = origin[0] + x as f32 * spacing;
                let py = origin[1] + y as f32 * spacing;
                let pz = origin[2] + z as f32 * spacing;
                // Signed distance to sphere of radius 0.7
                let dist = (px * px + py * py + pz * pz).sqrt() - 0.7;
                values.push(dist);
            }
        }
    }

    let volume = Volume {
        values,
        dimensions: [resolution, resolution, resolution],
        spacing: [spacing, spacing, spacing],
        origin,
    };

    // Extract isosurface at distance = 0
    println!("Extracting isosurface...");
    let mesh = marching_cubes(&volume, 0.0).with_material("sphere_mat");

    println!(
        "Generated mesh: {} vertices, {} triangles",
        mesh.positions.len() / 3,
        mesh.indices.len() / 3
    );

    // Create material for the sphere
    let sphere_material = Material::Solid(SolidMaterial::new("sphere_mat", [0.4, 0.6, 0.9]));

    // Create camera
    let camera = Camera::perspective([2.0, 1.5, 2.0], [0.0, 0.0, 0.0], 45.0);

    // Create scene
    let bounds = Bounds {
        min: [-1.5, -1.5, -1.5],
        max: [1.5, 1.5, 1.5],
    };
    let scene = Scene::new(camera, bounds)
        .add_material(sphere_material)
        .add_mesh(mesh);

    // Render
    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.05, 0.05, 0.1, 1.0],
    };

    println!("Rendering scene...");
    let png_data = render_to_png(&scene, &config).expect("Failed to render");

    // Write PNG
    fs::write("marching_cubes.png", &png_data).expect("Failed to write PNG");
    println!("Wrote marching_cubes.png ({} bytes)", png_data.len());
}
