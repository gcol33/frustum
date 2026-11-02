# Frustum

*3D scientific figures rendered on the GPU from an explicit scene*

<!-- badges: start -->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<!-- badges: end -->

Frustum renders 3D figures for papers, such as an isosurface of a scalar field, a point cloud coloured by a measurement, or a mesh with labelled axes, to a PNG file without a window.

It is a Rust workspace of two crates. `frustum-core` holds the scene model and the geometry with no GPU dependency, and `frustum-render` draws a scene through [wgpu](https://wgpu.rs), the Rust graphics library over Vulkan, Metal, DirectX 12 and OpenGL. A scene is one value: a camera with its position, target and projection, the scene bounds, the geometry elements and their materials, and an optional directional light. Nothing is inferred at render time, so the same scene and camera give the same figure, and the scene serialises to JSON for storing beside the figure or rebuilding it later. You build the scene in code, render it once, and write the bytes to disk.

Frustum supports:

- **Geometry.** Point clouds, polylines, triangle meshes and axis bundles with ticks and labels. Axes expand to lines and text before rendering.
- **Isosurfaces.** `marching_cubes()` turns a regular scalar volume into a mesh at one iso-value, with normals from the field gradient, and `marching_cubes_multi()` extracts several iso-values at once.
- **Colour.** A solid material or a scalar-mapped material with the viridis, plasma, inferno, magma or cividis colormap over an explicit range.
- **Shading.** One directional light with Lambertian shading, or flat colour when the scene has no light.
- **Render checks.** `render_with_audit()` returns the PNG together with an audit bundle: the GPU backend and adapter, a hash of the scene, primitive counts, image statistics and the result of invariant checks on the scene, camera, geometry and image.

The mathematical choices of the isosurface extraction (asymptotic decider for ambiguous cubes, linear interpolation in world space, gradient normals) are written down in `feature-planner/003-marching-cubes.md`, and the scene fields and their validation rules in `feature-planner/SCHEMA-INVENTORY.md`. Golden-image tests in `crates/frustum-render/tests/` compare a rendered cube and triangle against stored images within a per-pixel tolerance.

## Installation

Both crates are on crates.io. Add them to a Rust project with Cargo:

```sh
cargo add frustum-core frustum-render
```

The crates use the 2021 edition. Taking both from crates.io keeps one copy of `frustum-core` in the build, since `frustum-render` depends on the published one.

Rendering needs a GPU adapter that wgpu can open; without one `render_to_png()` returns `RenderError::AdapterCreation`. Rendering is headless and needs no window or display.

## Usage

1. Build the geometry, for example a mesh with `Mesh::new()` from flat position and index vectors.
2. Place a camera with `Camera::perspective()` or `Camera::orthographic()`.
3. Create the scene with `Scene::new()` from the camera and the scene bounds, and add the geometry.
4. Render with `render_to_png()` and write the bytes to a file.

```rust
use frustum_core::scene::{Bounds, Scene};
use frustum_core::{Camera, Mesh};
use frustum_render::{render_to_png, RenderConfig};

fn main() {
    let positions = vec![0.0, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, -0.5, 0.0];
    let mesh = Mesh::new(positions, vec![0, 1, 2]);

    let camera = Camera::perspective([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], 45.0);
    let bounds = Bounds { min: [-1.0, -1.0, -1.0], max: [1.0, 1.0, 1.0] };
    let scene = Scene::new(camera, bounds).add_mesh(mesh);

    let png = render_to_png(&scene, &RenderConfig::default()).unwrap();
    std::fs::write("triangle.png", png).unwrap();
}
```

Positions are x, y, z triples in a right-handed coordinate system with y up and the camera looking down its negative z axis. The default `RenderConfig` is 800 by 600 pixels on a white background. This scene has no light and no material, so the triangle is drawn in the default flat grey.

A material is added to the scene with `add_material()` and attached to an element by its id, as in `crates/frustum-render/examples/materials.rs`. A light is set with `with_light()`, as in `examples/lighting.rs`.

## Isosurfaces from a scalar volume

A `Volume` is a flat vector of values with its dimensions, voxel spacing and origin. `marching_cubes()` returns a `Mesh` that goes into the scene like any other mesh:

```rust
let volume = Volume::new(values, [32, 32, 32], [0.0645; 3], [-1.0; 3]);
let mesh = marching_cubes(&volume, 0.0);
```

The volume has to be at least 2 cells in each dimension. `crates/frustum-render/examples/marching_cubes.rs` builds the signed distance field of a sphere and renders its zero level set.

## Checking a render

`render_with_audit()` renders the scene and returns an `AuditBundle` beside the PNG. The bundle records which backend and adapter drew the image, the resolution, the camera, the scene bounds and primitive counts, and whether the invariant checks passed, for example that the geometry lies within the bounds and that the background colour reached the image. `compare_for_regression()` compares two bundles under a `RegressionTolerance`, which is how the golden tests detect a changed render. `examples/audit.rs` writes a bundle as JSON.

## Documentation

Start with the examples in `crates/frustum-render/examples/`: `cube.rs` for a mesh with a camera, `primitives.rs` for points and lines, `axes.rs` for an axis bundle, `materials.rs` for colormaps, `lighting.rs` for the directional light and `marching_cubes.rs` for an isosurface. Each runs with `cargo run -p frustum-render --example <name>` and writes a PNG into the working directory.

- [DESIGN.md](DESIGN.md): the frozen design principles, the coordinate system, the five primitives, the determinism guarantees and the non-goals.
- [PHILOSOPHY.md](PHILOSOPHY.md): why scene semantics are frozen before pixels.
- `feature-planner/`: one specification per feature, including the marching cubes commitments (003), the renderer contract (007) and the schema inventory.
- `schema/scene.schema.json`: the JSON schema a scene serialises to with `Scene::to_json()` and loads from with `Scene::from_json()`.
- API documentation builds locally with `cargo doc --open`.

## Getting help

Questions and bug reports go to the [GitHub issues](https://github.com/gcol33/frustum/issues). A bug report is easiest to act on with the scene as JSON, the output of `render_with_audit()` and the GPU backend it names.

## Support

> "Software is like sex: it's better when it's free." — Linus Torvalds

I'm a PhD student who builds tools in my free time because I believe good tools should be free and open. I started these projects for my own work and figured others might find them useful too.

If this package saved you some time, buying me a coffee is a nice way to say thanks. It helps with my coffee addiction.

[![Buy Me A Coffee](https://img.shields.io/badge/-Buy%20me%20a%20coffee-FFDD00?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/gcol33)

## License

Frustum is licensed under MIT.
