//! Golden tests for the rendering pipeline.
//!
//! These tests lock down the visual output of the renderer to prevent
//! unintended changes to the rendering behavior.

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{Camera, Mesh};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;
use std::path::PathBuf;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Create a deterministic cube mesh for testing.
fn test_cube_mesh() -> Mesh {
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

    Mesh::new(positions, indices)
}

/// Create a deterministic test scene with cube and camera.
fn test_cube_scene() -> Scene {
    let camera = Camera::perspective(
        [2.0, 1.5, 2.0],  // eye
        [0.0, 0.0, 0.0],  // target
        45.0,             // fov
    );

    let bounds = Bounds {
        min: [-1.0, -1.0, -1.0],
        max: [1.0, 1.0, 1.0],
    };

    Scene::new(camera, bounds).add_mesh(test_cube_mesh())
}

fn test_config() -> RenderConfig {
    RenderConfig {
        width: 256,
        height: 256,
        background: [0.1, 0.1, 0.15, 1.0],
    }
}

/// Compare two PNG images with tolerance.
/// Returns true if images are similar enough.
fn images_similar(a: &[u8], b: &[u8], tolerance: u8) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff_count = 0;
    let total_pixels = a.len() / 4;

    for i in (0..a.len()).step_by(4) {
        let dr = (a[i] as i32 - b[i] as i32).unsigned_abs();
        let dg = (a[i + 1] as i32 - b[i + 1] as i32).unsigned_abs();
        let db = (a[i + 2] as i32 - b[i + 2] as i32).unsigned_abs();

        if dr > tolerance as u32 || dg > tolerance as u32 || db > tolerance as u32 {
            diff_count += 1;
        }
    }

    // Allow up to 1% of pixels to differ (for cross-GPU variance)
    let max_diff = total_pixels / 100;
    diff_count <= max_diff
}

#[test]
fn test_cube_golden() {
    let _ = env_logger::builder().is_test(true).try_init();

    let scene = test_cube_scene();
    let config = test_config();

    let png_data = render_to_png(&scene, &config).expect("Failed to render");

    let golden_path = golden_dir().join("cube_256.png");

    if golden_path.exists() {
        // Compare with golden image
        let golden_data = fs::read(&golden_path).expect("Failed to read golden image");

        // Decode both PNGs to raw pixels for comparison
        let rendered_img = image::load_from_memory(&png_data).expect("Failed to decode rendered");
        let golden_img = image::load_from_memory(&golden_data).expect("Failed to decode golden");

        let rendered_rgba = rendered_img.to_rgba8();
        let golden_rgba = golden_img.to_rgba8();

        assert!(
            images_similar(rendered_rgba.as_raw(), golden_rgba.as_raw(), 5),
            "Rendered image differs from golden image. \
             If this is intentional, delete {} and re-run the test to update it.",
            golden_path.display()
        );
    } else {
        // Save as new golden image
        fs::write(&golden_path, &png_data).expect("Failed to write golden image");
        println!(
            "Created new golden image: {} ({} bytes)",
            golden_path.display(),
            png_data.len()
        );
    }
}

#[test]
fn test_triangle_golden() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = RenderConfig {
        width: 256,
        height: 256,
        background: [0.1, 0.1, 0.1, 1.0],
    };

    let png_data = frustum_render::render_test_triangle(&config).expect("Failed to render");

    let golden_path = golden_dir().join("triangle_256.png");

    if golden_path.exists() {
        let golden_data = fs::read(&golden_path).expect("Failed to read golden image");

        let rendered_img = image::load_from_memory(&png_data).expect("Failed to decode rendered");
        let golden_img = image::load_from_memory(&golden_data).expect("Failed to decode golden");

        let rendered_rgba = rendered_img.to_rgba8();
        let golden_rgba = golden_img.to_rgba8();

        assert!(
            images_similar(rendered_rgba.as_raw(), golden_rgba.as_raw(), 5),
            "Rendered image differs from golden image. \
             If this is intentional, delete {} and re-run the test to update it.",
            golden_path.display()
        );
    } else {
        fs::write(&golden_path, &png_data).expect("Failed to write golden image");
        println!(
            "Created new golden image: {} ({} bytes)",
            golden_path.display(),
            png_data.len()
        );
    }
}
