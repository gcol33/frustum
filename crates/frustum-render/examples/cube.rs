//! Render a cube using the full scene/camera pipeline
//!
//! Run with: cargo run -p frustum-render --example cube

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{Camera, Mesh};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    // Create a cube mesh
    // Vertices with positions (we'll use a simple colored cube)
    #[rustfmt::skip]
    let positions: Vec<f32> = vec![
        // Front face
        -0.5, -0.5,  0.5,
         0.5, -0.5,  0.5,
         0.5,  0.5,  0.5,
        -0.5,  0.5,  0.5,
        // Back face
        -0.5, -0.5, -0.5,
        -0.5,  0.5, -0.5,
         0.5,  0.5, -0.5,
         0.5, -0.5, -0.5,
        // Top face
        -0.5,  0.5, -0.5,
        -0.5,  0.5,  0.5,
         0.5,  0.5,  0.5,
         0.5,  0.5, -0.5,
        // Bottom face
        -0.5, -0.5, -0.5,
         0.5, -0.5, -0.5,
         0.5, -0.5,  0.5,
        -0.5, -0.5,  0.5,
        // Right face
         0.5, -0.5, -0.5,
         0.5,  0.5, -0.5,
         0.5,  0.5,  0.5,
         0.5, -0.5,  0.5,
        // Left face
        -0.5, -0.5, -0.5,
        -0.5, -0.5,  0.5,
        -0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
    ];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0,  1,  2,  0,  2,  3,   // front
        4,  5,  6,  4,  6,  7,   // back
        8,  9,  10, 8,  10, 11,  // top
        12, 13, 14, 12, 14, 15,  // bottom
        16, 17, 18, 16, 18, 19,  // right
        20, 21, 22, 20, 22, 23,  // left
    ];

    let mesh = Mesh::new(positions, indices);

    // Create camera looking at the cube from an angle
    let camera = Camera::perspective(
        [2.0, 1.5, 2.0],  // eye position
        [0.0, 0.0, 0.0],  // target (center of cube)
        45.0,             // field of view
    );

    // Create scene
    let bounds = Bounds {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };
    let scene = Scene::new(camera, bounds).add_mesh(mesh);

    // Render
    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.1, 0.1, 0.15, 1.0],
    };

    println!("Rendering cube scene...");
    let png_data = render_to_png(&scene, &config).expect("Failed to render");

    let output_path = "cube.png";
    fs::write(output_path, &png_data).expect("Failed to write PNG");
    println!("Wrote {} bytes to {}", png_data.len(), output_path);
}
