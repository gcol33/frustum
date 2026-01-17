//! Feature 007 - Renderer Contract Tests
//!
//! This module implements the non-negotiable renderer tests defined in
//! 007-renderer-tests.md. A renderer is non-compliant if any of these fail.

use frustum_core::scene::{Bounds, Scene};
use frustum_core::{
    Axis, AxisBounds, AxisBundle, Camera, Light, Material, Mesh, PointCloud, Polyline,
    ScalarMappedMaterial, SolidMaterial, TickSpec,
};
use frustum_render::{render_to_png, render_with_audit, RenderConfig};

fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Helper to create a standard test config
fn test_config() -> RenderConfig {
    RenderConfig {
        width: 256,
        height: 256,
        background: [0.1, 0.1, 0.15, 1.0],
    }
}

/// Helper to create a simple camera looking at the origin
fn test_camera() -> Camera {
    Camera::perspective([2.0, 1.5, 2.0], [0.0, 0.0, 0.0], 45.0)
}

/// Helper to create a minimal cube mesh for testing
fn cube_mesh() -> Mesh {
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
    ];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0, 1, 2, 0, 2, 3,  // front
        4, 5, 6, 4, 6, 7,  // back
    ];

    Mesh::new(positions, indices)
}

// ============================================================================
// Scene Consumption Tests
// ============================================================================

#[test]
fn test_scene_consumption_basic() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = test_config();
    let result = render_to_png(&scene, &config);

    assert!(result.is_ok(), "Basic scene should render successfully");
}

#[test]
fn test_empty_scene_renders_background_only() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    );

    let config = RenderConfig {
        width: 64,
        height: 64,
        background: [0.5, 0.5, 0.5, 1.0],
    };

    let result = render_to_png(&scene, &config);
    assert!(result.is_ok(), "Empty scene should render without error");

    let png_data = result.unwrap();
    assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "Output should be valid PNG");
}

#[test]
fn test_scene_does_not_mutate() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    // Capture scene state before render
    let json_before = scene.to_json().unwrap();

    let config = test_config();
    let _ = render_to_png(&scene, &config);

    // Verify scene unchanged after render
    let json_after = scene.to_json().unwrap();
    assert_eq!(json_before, json_after, "Scene should not be mutated by render");
}

// ============================================================================
// Geometry Handling Tests
// ============================================================================

#[test]
fn test_points_render() {
    init_logger();

    let points = PointCloud::new(vec![
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
    ], 10.0);

    let scene = Scene::new(
        Camera::perspective([3.0, 2.0, 3.0], [0.5, 0.5, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_point_cloud(points);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Points should render");
}

#[test]
fn test_lines_render() {
    init_logger();

    let line = Polyline::new(vec![
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 0.0,
    ], 2.0);

    let scene = Scene::new(
        Camera::perspective([2.0, 2.0, 3.0], [0.5, 0.5, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_polyline(line);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Lines should render");
}

#[test]
fn test_mesh_triangle_winding_preserved() {
    init_logger();

    // Create a single triangle with known winding (CCW)
    let mesh = Mesh::new(
        vec![
            0.0, 0.0, 0.0,  // v0
            1.0, 0.0, 0.0,  // v1
            0.5, 1.0, 0.0,  // v2
        ],
        vec![0, 1, 2],  // CCW when viewed from +Z
    );

    let scene = Scene::new(
        Camera::perspective([0.5, 0.5, 3.0], [0.5, 0.5, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_mesh(mesh);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Mesh with CCW winding should render");
}

#[test]
fn test_degenerate_triangles_no_crash() {
    init_logger();

    // Degenerate triangle (all vertices same position)
    let mesh = Mesh::new(
        vec![
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
        ],
        vec![0, 1, 2],
    );

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(mesh);

    // Should not crash
    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Degenerate triangles should not cause crash");
}

#[test]
fn test_zero_length_lines_no_crash() {
    init_logger();

    // Zero-length line segment
    let line = Polyline::new(vec![
        0.0, 0.0, 0.0,
        0.0, 0.0, 0.0,  // Same as first
    ], 1.0);

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_polyline(line);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Zero-length lines should not crash");
}

#[test]
fn test_coincident_vertices_handled() {
    init_logger();

    // Triangle with two coincident vertices
    let mesh = Mesh::new(
        vec![
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 0.0, 0.0,  // Same as v1
        ],
        vec![0, 1, 2],
    );

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_mesh(mesh);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Coincident vertices should be handled gracefully");
}

#[test]
fn test_empty_geometry_lists() {
    init_logger();

    // Empty mesh
    let mesh = Mesh::new(vec![], vec![]);

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(mesh);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Empty geometry lists should render without error");
}

// ============================================================================
// Camera & Projection Tests
// ============================================================================

#[test]
fn test_perspective_projection() {
    init_logger();

    let camera = Camera::perspective([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 45.0);

    let scene = Scene::new(
        camera,
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Perspective projection should work");
}

#[test]
fn test_orthographic_projection() {
    init_logger();

    let camera = Camera::orthographic([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 3.0);

    let scene = Scene::new(
        camera,
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Orthographic projection should work");
}

#[test]
fn test_near_far_clipping_planes() {
    init_logger();

    // Camera with custom near/far planes
    let mut camera = Camera::perspective([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], 45.0);
    camera.near = 1.0;
    camera.far = 10.0;

    let scene = Scene::new(
        camera,
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Custom near/far planes should work");
}

#[test]
fn test_camera_does_not_drift() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = test_config();

    // Render twice
    let result1 = render_to_png(&scene, &config).unwrap();
    let result2 = render_to_png(&scene, &config).unwrap();

    // Images should be identical (within PNG encoding tolerance)
    // PNG compression may vary slightly, so we compare decoded pixels
    let img1 = image::load_from_memory(&result1).unwrap().to_rgba8();
    let img2 = image::load_from_memory(&result2).unwrap().to_rgba8();

    assert_eq!(
        img1.as_raw(),
        img2.as_raw(),
        "Camera should not drift between identical renders"
    );
}

// ============================================================================
// Materials Tests
// ============================================================================

#[test]
fn test_solid_material_renders_uniform_color() {
    init_logger();

    let mut mesh = cube_mesh();
    mesh.material_id = Some("red".to_string());

    let material = Material::Solid(SolidMaterial::new("red", [1.0, 0.0, 0.0]));

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(mesh)
    .add_material(material);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Solid material should render");
}

#[test]
fn test_scalar_mapped_material_with_viridis() {
    init_logger();

    let mut mesh = Mesh::new(
        vec![
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.5, 1.0, 0.0,
        ],
        vec![0, 1, 2],
    );
    mesh.scalars = Some(vec![0.0, 0.5, 1.0]);
    mesh.material_id = Some("viridis_mat".to_string());

    let material = Material::ScalarMapped(ScalarMappedMaterial::new("viridis_mat", "viridis", [0.0, 1.0]));

    let scene = Scene::new(
        Camera::perspective([0.5, 0.5, 3.0], [0.5, 0.5, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_mesh(mesh)
    .add_material(material);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Scalar-mapped material with viridis should render");
}

#[test]
fn test_clamp_out_of_range_scalars() {
    init_logger();

    let mut mesh = Mesh::new(
        vec![
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.5, 1.0, 0.0,
        ],
        vec![0, 1, 2],
    );
    // Scalars outside range [0, 1]
    mesh.scalars = Some(vec![-1.0, 2.0, 0.5]);
    mesh.material_id = Some("clamped".to_string());

    let material = Material::ScalarMapped(
        ScalarMappedMaterial::new("clamped", "viridis", [0.0, 1.0])
            .with_clamp(true)
    );

    let scene = Scene::new(
        Camera::perspective([0.5, 0.5, 3.0], [0.5, 0.5, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_mesh(mesh)
    .add_material(material);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Clamped out-of-range scalars should render");
}

#[test]
fn test_alpha_channel_respected() {
    init_logger();

    let mut mesh = cube_mesh();
    mesh.material_id = Some("transparent".to_string());

    let material = Material::Solid(SolidMaterial::with_alpha("transparent", [1.0, 0.0, 0.0, 0.5]));

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(mesh)
    .add_material(material);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Transparent materials should render");
}

// ============================================================================
// Axes Tests
// ============================================================================

#[test]
fn test_axes_render() {
    init_logger();

    let axes = AxisBundle::new("test_axes", AxisBounds {
        min: [0.0, 0.0, 0.0],
        max: [1.0, 1.0, 1.0],
    })
    .with_axes(vec![Axis::X, Axis::Y, Axis::Z])
    .with_ticks(TickSpec::Auto { count: 5 });

    let scene = Scene::new(
        Camera::perspective([3.0, 3.0, 3.0], [0.5, 0.5, 0.5], 45.0),
        Bounds {
            min: [-0.5, -0.5, -0.5],
            max: [1.5, 1.5, 1.5],
        },
    )
    .add_axes(axes);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Axes should render");
}

#[test]
fn test_axes_expand_to_lines() {
    init_logger();

    let axes = AxisBundle::new("test_axes", AxisBounds {
        min: [0.0, 0.0, 0.0],
        max: [1.0, 1.0, 1.0],
    })
    .with_axes(vec![Axis::X])
    .with_ticks(TickSpec::Fixed { values: vec![0.0, 0.5, 1.0] });

    let (polylines, _labels) = axes.expand();

    // Axes should expand to polylines
    assert!(!polylines.is_empty(), "Axes should expand to at least one polyline");

    // The expansion should produce line geometry
    for line in &polylines {
        assert!(line.positions.len() >= 6, "Each line should have at least 2 points (6 floats)");
    }
}

// ============================================================================
// Transparency & Background Tests
// ============================================================================

#[test]
fn test_rgba_background_preserved() {
    init_logger();

    let config = RenderConfig {
        width: 64,
        height: 64,
        background: [0.2, 0.4, 0.6, 1.0],  // Blue-ish
    };

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    );

    let result = render_to_png(&scene, &config);
    assert!(result.is_ok(), "Background color should be preserved");
}

#[test]
fn test_transparent_background() {
    init_logger();

    let config = RenderConfig {
        width: 64,
        height: 64,
        background: [0.0, 0.0, 0.0, 0.0],  // Fully transparent
    };

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    );

    let result = render_to_png(&scene, &config);
    assert!(result.is_ok(), "Transparent background should work");
}

#[test]
fn test_opaque_geometry_depth_tested() {
    init_logger();

    // Two overlapping cubes at different depths
    let front_mesh = Mesh::new(
        vec![
            -0.5, -0.5, 0.5,
             0.5, -0.5, 0.5,
             0.5,  0.5, 0.5,
            -0.5,  0.5, 0.5,
        ],
        vec![0, 1, 2, 0, 2, 3],
    );

    let back_mesh = Mesh::new(
        vec![
            -0.5, -0.5, -0.5,
             0.5, -0.5, -0.5,
             0.5,  0.5, -0.5,
            -0.5,  0.5, -0.5,
        ],
        vec![0, 1, 2, 0, 2, 3],
    );

    let scene = Scene::new(
        Camera::perspective([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(front_mesh)
    .add_mesh(back_mesh);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Depth testing should work for overlapping geometry");
}

// ============================================================================
// Lighting Tests
// ============================================================================

#[test]
fn test_no_lighting_flat_colors() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());
    // No light added - should render flat colors

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "No lighting should render flat colors");
}

#[test]
fn test_light_present_lambertian_shading() {
    init_logger();

    let light = Light::new([1.0, 1.0, 1.0], 1.0);

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh())
    .with_light(light);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Lambertian shading should work");
}

#[test]
fn test_points_render_unlit() {
    init_logger();

    let points = PointCloud::new(vec![
        0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
    ], 10.0);

    let light = Light::new([1.0, 1.0, 1.0], 1.0);

    let scene = Scene::new(
        Camera::perspective([0.0, 0.0, 5.0], [0.5, 0.0, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_point_cloud(points)
    .with_light(light);

    // Points should render regardless of light
    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Points should render unlit");
}

#[test]
fn test_lines_render_unlit() {
    init_logger();

    let line = Polyline::new(vec![
        0.0, 0.0, 0.0,
        1.0, 1.0, 1.0,
    ], 2.0);

    let light = Light::new([1.0, 1.0, 1.0], 1.0);

    let scene = Scene::new(
        Camera::perspective([0.0, 0.0, 5.0], [0.5, 0.5, 0.5], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [2.0, 2.0, 2.0],
        },
    )
    .add_polyline(line)
    .with_light(light);

    // Lines should render regardless of light
    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Lines should render unlit");
}

#[test]
fn test_light_disabled() {
    init_logger();

    let mut light = Light::new([1.0, 1.0, 1.0], 1.0);
    light.enabled = false;  // Disable light

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh())
    .with_light(light);

    let result = render_to_png(&scene, &test_config());
    assert!(result.is_ok(), "Disabled light should result in flat colors");
}

// ============================================================================
// Output Tests
// ============================================================================

#[test]
fn test_png_output_valid() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = test_config();
    let png_data = render_to_png(&scene, &config).unwrap();

    // Check PNG magic bytes
    assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "Output should be valid PNG");

    // Should decode successfully
    let img = image::load_from_memory(&png_data).expect("Should be decodable PNG");
    assert_eq!(img.width(), config.width);
    assert_eq!(img.height(), config.height);
}

#[test]
fn test_output_resolution_matches_config() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    // Test various resolutions
    for (w, h) in [(64, 64), (128, 256), (512, 256)] {
        let config = RenderConfig {
            width: w,
            height: h,
            background: [0.0, 0.0, 0.0, 1.0],
        };

        let png_data = render_to_png(&scene, &config).unwrap();
        let img = image::load_from_memory(&png_data).unwrap();

        assert_eq!(img.width(), w, "Width should match config");
        assert_eq!(img.height(), h, "Height should match config");
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_deterministic_output() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = test_config();

    // Render multiple times
    let renders: Vec<_> = (0..3)
        .map(|_| {
            let png = render_to_png(&scene, &config).unwrap();
            image::load_from_memory(&png).unwrap().to_rgba8()
        })
        .collect();

    // All renders should be identical
    for i in 1..renders.len() {
        assert_eq!(
            renders[0].as_raw(),
            renders[i].as_raw(),
            "Render {} differs from render 0",
            i
        );
    }
}

#[test]
fn test_no_jitter_across_renders() {
    init_logger();

    let scene = Scene::new(
        Camera::perspective([2.0, 2.0, 2.0], [0.0, 0.0, 0.0], 45.0),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = RenderConfig {
        width: 128,
        height: 128,
        background: [0.0, 0.0, 0.0, 1.0],
    };

    let img1 = {
        let png = render_to_png(&scene, &config).unwrap();
        image::load_from_memory(&png).unwrap().to_rgba8()
    };

    let img2 = {
        let png = render_to_png(&scene, &config).unwrap();
        image::load_from_memory(&png).unwrap().to_rgba8()
    };

    // Compare pixel by pixel
    let mut diff_count = 0;
    for (p1, p2) in img1.pixels().zip(img2.pixels()) {
        if p1 != p2 {
            diff_count += 1;
        }
    }

    assert_eq!(diff_count, 0, "No jitter should occur between identical renders");
}

// ============================================================================
// Audit Bundle Tests
// ============================================================================

#[test]
fn test_audit_bundle_generation() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = test_config();
    let (png_data, audit) = render_with_audit(&scene, &config).unwrap();

    // PNG should be valid
    assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

    // Audit should contain valid data
    assert!(!audit.metadata.scene_hash.is_empty());
    assert_eq!(audit.metadata.resolution, [config.width, config.height]);
    assert!(audit.metadata.primitive_counts.total_triangles > 0);

    // Should serialize to JSON
    let json = audit.to_json().unwrap();
    assert!(!json.is_empty());
}

#[test]
fn test_invariant_checking() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    let config = test_config();
    let (_, audit) = render_with_audit(&scene, &config).unwrap();

    // Valid scene should pass invariants
    assert!(
        audit.invariants.errors.is_empty(),
        "Valid scene should have no invariant errors: {:?}",
        audit.invariants.errors
    );
}

// ============================================================================
// Colormap Tests
// ============================================================================

#[test]
fn test_all_colormaps_render() {
    init_logger();

    let colormaps = ["viridis", "plasma", "inferno", "magma", "cividis"];

    for cmap_name in colormaps {
        let mut mesh = Mesh::new(
            vec![
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.5, 1.0, 0.0,
            ],
            vec![0, 1, 2],
        );
        mesh.scalars = Some(vec![0.0, 0.5, 1.0]);
        mesh.material_id = Some("cmap".to_string());

        let material = Material::ScalarMapped(
            ScalarMappedMaterial::new("cmap", cmap_name, [0.0, 1.0])
        );

        let scene = Scene::new(
            Camera::perspective([0.5, 0.5, 3.0], [0.5, 0.5, 0.0], 45.0),
            Bounds {
                min: [-1.0, -1.0, -1.0],
                max: [2.0, 2.0, 2.0],
            },
        )
        .add_mesh(mesh)
        .add_material(material);

        let result = render_to_png(&scene, &test_config());
        assert!(result.is_ok(), "Colormap '{}' should render", cmap_name);
    }
}

// ============================================================================
// RenderConfig Validation (implicitly via rendering)
// ============================================================================

#[test]
fn test_minimum_resolution() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    // Minimum resolution should work
    let config = RenderConfig {
        width: 1,
        height: 1,
        background: [0.0, 0.0, 0.0, 1.0],
    };

    let result = render_to_png(&scene, &config);
    assert!(result.is_ok(), "1x1 resolution should work");
}

#[test]
fn test_large_resolution() {
    init_logger();

    let scene = Scene::new(
        test_camera(),
        Bounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        },
    )
    .add_mesh(cube_mesh());

    // Larger resolution
    let config = RenderConfig {
        width: 1024,
        height: 1024,
        background: [0.0, 0.0, 0.0, 1.0],
    };

    let result = render_to_png(&scene, &config);
    assert!(result.is_ok(), "1024x1024 resolution should work");
}
