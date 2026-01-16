# Frustum

A deterministic, GPU-first 3D rendering framework for scientific figures.

## What Frustum Is

Frustum provides Matplotlib-grade 3D capability with a modern rendering architecture, explicit scene semantics, and cross-language parity between Python and R.

## What Frustum Is Not

- Not a plotting DSL
- Not a dashboard tool
- Not a visualization playground

## Design Principles

- **Explicit scene graph** — no implicit state
- **Deterministic intent** — perceptually stable output
- **Static rendering first** — interaction optional
- **GPU required** — no legacy backends
- **Cross-language parity** — identical feature surface for Python and R
- **Narrow scope** — long-term stability over features
- **Early errors** — explicit and informative

## Features

### Scene Model
- Scene as a single immutable object
- Explicit camera definition (position, target, up vector, projection type, near/far planes)
- Explicit coordinate system and scene bounds

### Geometry Primitives
- Point clouds (per-point position, scalar colormap, uniform size)
- Line geometry (polylines, uniform width)
- Triangle meshes (indexed triangles, optional normals, optional per-vertex scalars)

### Marching Cubes
- Regular 3D scalar volume input
- Configurable voxel spacing and origin
- Single or multiple iso-levels
- Stable, well-defined normals
- Optional pre-smoothing and mesh decimation

### Materials and Color
- Shared colormap system (Matplotlib-compatible semantics)
- Scalar-to-color mapping
- Simple Lambertian shading with directional/headlight

### Axes
- Optional axes as explicit geometry
- Deterministic tick marks and labels

## Architecture

- **Core**: Rust + wgpu + WGSL shaders
- **Frontends**: Python and R (feature-equivalent)
- **Schema**: TypeScript-defined canonical scene schema with JSON derivation
- **Output**: Headless PNG rendering at fixed resolution/DPI

## Determinism Guarantees

Frustum guarantees:
- Same scene + same topology + same camera = same figure meaning
- No hidden state
- No order-dependent rendering (unless explicitly documented)

Frustum does not guarantee:
- Bitwise identical pixels across GPUs

## Installation

*Coming soon*

## License

MIT
