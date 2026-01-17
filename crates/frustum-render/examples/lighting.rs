//! Demonstrate the lighting system with Lambertian shading.
//!
//! Shows the difference between lit and unlit scenes, and
//! how light direction affects mesh shading.
//!
//! Run with: cargo run -p frustum-render --example lighting

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{marching_cubes, Camera, Light, Material, SolidMaterial, Volume};
use frustum_render::{render_to_png, RenderConfig};
use std::fs;

fn create_sphere_mesh() -> frustum_core::Mesh {
    // Create sphere using marching cubes for smooth normals
    let resolution = 24;
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
                let dist = (px * px + py * py + pz * pz).sqrt() - 0.6;
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

    marching_cubes(&volume, 0.0)
}

fn main() {
    env_logger::init();

    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.05, 0.05, 0.1, 1.0],
    };

    let bounds = Bounds {
        min: [-1.5, -1.5, -1.5],
        max: [1.5, 1.5, 1.5],
    };

    let camera = Camera::perspective([1.5, 1.0, 1.5], [0.0, 0.0, 0.0], 45.0);
    let sphere = create_sphere_mesh().with_material("sphere_mat");
    let material = Material::Solid(SolidMaterial::new("sphere_mat", [0.6, 0.7, 0.9]));

    // Scene 1: No light (flat colors)
    println!("Rendering scene without lighting (flat colors)...");
    let scene_flat = Scene::new(camera.clone(), bounds)
        .add_material(material.clone())
        .add_mesh(sphere.clone());

    let png_data = render_to_png(&scene_flat, &config).expect("Failed to render");
    fs::write("lighting_flat.png", &png_data).expect("Failed to write PNG");
    println!("Wrote lighting_flat.png ({} bytes)", png_data.len());

    // Scene 2: Light from upper-right-front
    println!("Rendering scene with light from upper-right-front...");
    let light_front = Light::new([1.0, 1.0, 1.0], 1.0);
    let scene_lit = Scene::new(camera.clone(), bounds)
        .add_material(material.clone())
        .add_mesh(sphere.clone())
        .with_light(light_front);

    let png_data = render_to_png(&scene_lit, &config).expect("Failed to render");
    fs::write("lighting_front.png", &png_data).expect("Failed to write PNG");
    println!("Wrote lighting_front.png ({} bytes)", png_data.len());

    // Scene 3: Light from above
    println!("Rendering scene with light from above...");
    let light_top = Light::new([0.0, 1.0, 0.0], 1.0);
    let scene_top = Scene::new(camera.clone(), bounds)
        .add_material(material.clone())
        .add_mesh(sphere.clone())
        .with_light(light_top);

    let png_data = render_to_png(&scene_top, &config).expect("Failed to render");
    fs::write("lighting_top.png", &png_data).expect("Failed to write PNG");
    println!("Wrote lighting_top.png ({} bytes)", png_data.len());

    // Scene 4: Light from the side with higher intensity
    println!("Rendering scene with bright side light...");
    let light_side = Light::new([1.0, 0.0, 0.0], 1.5);
    let scene_side = Scene::new(camera, bounds)
        .add_material(material)
        .add_mesh(sphere)
        .with_light(light_side);

    let png_data = render_to_png(&scene_side, &config).expect("Failed to render");
    fs::write("lighting_side.png", &png_data).expect("Failed to write PNG");
    println!("Wrote lighting_side.png ({} bytes)", png_data.len());

    println!("\nLighting examples complete!");
    println!("  - lighting_flat.png: No lighting (flat material color)");
    println!("  - lighting_front.png: Light from upper-right-front");
    println!("  - lighting_top.png: Light from directly above");
    println!("  - lighting_side.png: Bright light from the side");
}
