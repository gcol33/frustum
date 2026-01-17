//! Render a test triangle to PNG
//!
//! Run with: cargo run -p frustum-render --example triangle

use frustum_render::{render_test_triangle, RenderConfig};
use std::fs;

fn main() {
    env_logger::init();

    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.1, 0.1, 0.15, 1.0],
    };

    println!("Rendering test triangle...");
    let png_data = render_test_triangle(&config).expect("Failed to render");

    let output_path = "triangle.png";
    fs::write(output_path, &png_data).expect("Failed to write PNG");
    println!("Wrote {} bytes to {}", png_data.len(), output_path);
}
