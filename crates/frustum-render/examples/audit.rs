//! Render a cube and produce an audit bundle for sanity checking.
//!
//! Run with: cargo run -p frustum-render --example audit

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{Camera, Mesh};
use frustum_render::{render_with_audit, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    // Create a cube mesh
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

    // Create camera
    let camera = Camera::perspective([2.0, 1.5, 2.0], [0.0, 0.0, 0.0], 45.0);

    // Create scene
    let bounds = Bounds {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };
    let scene = Scene::new(camera, bounds).add_mesh(mesh);

    // Render with audit
    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.1, 0.1, 0.15, 1.0],
    };

    println!("Rendering cube with audit...");
    let (png_data, audit) = render_with_audit(&scene, &config).expect("Failed to render");

    // Write PNG
    fs::write("audit_cube.png", &png_data).expect("Failed to write PNG");
    println!("Wrote audit_cube.png ({} bytes)", png_data.len());

    // Write audit bundle as JSON
    let audit_json = audit.to_json().expect("Failed to serialize audit");
    fs::write("audit_cube.json", &audit_json).expect("Failed to write audit JSON");
    println!("Wrote audit_cube.json ({} bytes)", audit_json.len());

    // Print summary
    println!("\n=== Audit Summary ===");
    println!("Scene hash: {}", audit.metadata.scene_hash);
    println!("Backend: {}", audit.metadata.backend);
    println!("Resolution: {}x{}", audit.metadata.resolution[0], audit.metadata.resolution[1]);
    println!(
        "Primitives: {} meshes, {} triangles",
        audit.metadata.primitive_counts.meshes,
        audit.metadata.primitive_counts.total_triangles
    );
    println!(
        "Image: {:.1}% background, {:.3} edge density",
        audit.image_metrics.background_percentage,
        audit.image_metrics.edge_density
    );
    println!("\n=== Invariant Results ===");
    println!("Status: {:?}", audit.invariants.overall);
    for err in &audit.invariants.errors {
        println!("ERROR [{}]: {}", err.category, err.message);
    }
    for warn in &audit.invariants.warnings {
        println!("WARNING [{}]: {}", warn.category, warn.message);
    }
    for note in &audit.invariants.notes {
        println!("NOTE: {}", note);
    }
}
