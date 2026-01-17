# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Frustum is a deterministic, GPU-first 3D rendering framework for scientific figures with cross-language parity between Python and R. The core is written in Rust using wgpu for GPU rendering.

## Build Commands

```bash
# Build the entire workspace
cargo build

# Build and run tests
cargo test

# Run a specific crate's tests
cargo test -p frustum-core
cargo test -p frustum-render

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture

### Crate Structure

- **frustum-core**: Scene model, camera, and geometry primitives (no GPU dependencies)
  - `camera.rs`: Camera with explicit position/target/up, perspective/orthographic projection
  - `geometry.rs`: PointCloud, Polyline, Mesh primitives with optional scalars
  - `scene.rs`: Scene container with SceneElement enum, JSON serialization

- **frustum-render**: wgpu-based GPU rendering backend (depends on frustum-core)
  - `render_to_png()`: Main entry point for headless rendering
  - Currently a scaffold awaiting implementation

### Data Flow

```
Scene (JSON) → frustum-core (validation) → frustum-render (wgpu) → PNG
```

### Schema

- Canonical JSON schema: `schema/scene.schema.json`
- Full field inventory: `feature-planner/SCHEMA-INVENTORY.md`

## Design Constraints

These are non-negotiable principles from DESIGN.md:

- **Explicit scene graph**: No implicit state
- **Deterministic intent**: Same scene + camera = same figure meaning
- **GPU required**: No software fallbacks
- **Cross-language parity**: Python and R frontends must be feature-equivalent
- **Primitive rule (frozen)**: If expressible as existing primitives, not a new primitive

### Coordinate System

OpenGL convention (right-handed):
- +X → right, +Y → up, −Z → forward (into scene), +Z → toward viewer

### Five Geometry Primitives (Complete Set)

1. Points — 0D, positions + optional scalars + uniform size
2. Lines — 1D piecewise linear, positions + uniform width
3. Curves — 1D parametric (Bézier/Catmull-Rom/B-spline), evaluated to Lines
4. Meshes — 2D triangles, indexed with optional normals/scalars
5. Axes — reference geometry, expanded to Lines before rendering

## Marching Cubes Mathematical Commitments

Feature 003 freezes these choices:

- **Disambiguation**: Asymptotic decider for saddle cases
- **Interpolation**: Linear, world-space, exact formula frozen
- **Normals**: Gradient-based (central differences), not face-based
- **Preconditions**: Continuous scalar field, band-limited (Nyquist)

Categorical volumes are rejected unless explicitly opted in.

## Feature Planning

Detailed feature specifications in `feature-planner/`:
- 001-008: Frozen v0.1 features
- SCHEMA-INVENTORY.md: Complete field ownership and validation rules
- 009-extensions.md: Future directions (informational only)
