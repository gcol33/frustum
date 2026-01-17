//! Example showcasing 009 extensions: multi-isovalue, derived scalars, lighting presets.
//!
//! Creates nested spherical shells with gradient-magnitude coloring.

use frustum_core::{
    marching_cubes_multi, Axis, AxisBounds, AxisBundle, Camera, Light,
    Material, SolidMaterial, TickSpec, Volume,
};
use frustum_core::scene::{Bounds, Scene};
use frustum_render::{render_to_png, RenderConfig};

fn main() {
    println!("Rendering extensions example...");

    // Create a 3D volume with a sphere SDF
    let size = 40;
    let mut values = Vec::with_capacity(size * size * size);

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let fx = (x as f32 / (size - 1) as f32) * 2.0 - 1.0;
                let fy = (y as f32 / (size - 1) as f32) * 2.0 - 1.0;
                let fz = (z as f32 / (size - 1) as f32) * 2.0 - 1.0;
                // Signed distance to sphere of radius 1.0 at origin
                let dist = (fx * fx + fy * fy + fz * fz).sqrt() - 0.9;
                values.push(dist);
            }
        }
    }

    let volume = Volume::new(
        values,
        [size, size, size],
        [2.0 / (size - 1) as f32; 3],
        [-1.0, -1.0, -1.0],
    );

    // Extension 4: Multi-isovalue extraction - create nested shells
    let iso_values = [-0.3, 0.0, 0.3];
    let surfaces = marching_cubes_multi(&volume, &iso_values);

    println!("Extracted {} isosurfaces:", surfaces.len());
    for (i, surface) in surfaces.iter().enumerate() {
        let tri_count = surface.mesh.indices.len() / 3;
        println!("  iso={:.1}: {} triangles", surface.iso_value, tri_count);
    }

    // Extension 5: Compute gradient magnitude for feature highlighting
    let grad_mag = volume.gradient_magnitude();
    let (min, max) = grad_mag.value_range();
    println!("Gradient magnitude range: [{:.3}, {:.3}]", min, max);

    // Create scene with camera
    let camera = Camera::perspective([2.5, 2.0, 2.5], [0.0, 0.0, 0.0], 45.0);

    let mut scene = Scene::new(
        camera,
        Bounds {
            min: [-1.2, -1.2, -1.2],
            max: [1.2, 1.2, 1.2],
        },
    );

    // Add materials for each shell
    let colors = [
        [0.2, 0.4, 0.8], // Inner: blue
        [0.8, 0.3, 0.3], // Middle: red
        [0.3, 0.8, 0.4], // Outer: green
    ];

    for (i, surface) in surfaces.iter().enumerate() {
        if !surface.mesh.positions.is_empty() {
            let material_id = format!("shell_{}", i);
            scene = scene.add_material(Material::Solid(
                SolidMaterial::with_alpha(&material_id, [colors[i][0], colors[i][1], colors[i][2], 0.7]),
            ));

            let mut mesh = surface.mesh.clone();
            mesh.material_id = Some(material_id);
            scene = scene.add_mesh(mesh);
        }
    }

    // Add coordinate axes
    scene = scene.add_axes(
        AxisBundle::new(
            "axes",
            AxisBounds {
                min: [-1.0, -1.0, -1.0],
                max: [1.0, 1.0, 1.0],
            },
        )
        .with_axes(vec![Axis::X, Axis::Y, Axis::Z])
        .with_ticks(TickSpec::Auto { count: 3 }),
    );

    // Lighting presets comparison
    let presets: Vec<(&str, Light)> = vec![
        ("scientific_flat", Light::scientific_flat()),
        ("studio_soft", Light::studio_soft()),
        ("rim_highlight", Light::rim_highlight()),
        ("depth_emphasis", Light::depth_emphasis()),
        ("side_light", Light::side_light()),
        ("three_quarter", Light::three_quarter()),
    ];

    let config = RenderConfig {
        width: 512,
        height: 512,
        background: [0.1, 0.1, 0.15, 1.0],
    };

    // Render with each lighting preset
    for (name, light) in presets {
        let mut scene_with_light = scene.clone();
        scene_with_light.light = Some(light);

        match render_to_png(&scene_with_light, &config) {
            Ok(png_data) => {
                let filename = format!("extensions_{}.png", name);
                std::fs::write(&filename, &png_data).expect("Failed to write PNG");
                println!("Wrote {} ({} bytes)", filename, png_data.len());
            }
            Err(e) => eprintln!("Render error with {}: {:?}", name, e),
        }
    }

    println!("\nDone! Check the extensions_*.png files for lighting comparison.");
}
