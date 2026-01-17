# Frustum Design Document

## Purpose

Frustum is a deterministic, GPU-first 3D rendering framework for scientific figures.
It is not a plotting DSL, not a dashboard tool, and not a visualization playground.

The goal is to provide Matplotlib-grade 3D capability with a modern rendering architecture, explicit scene semantics, and cross-language parity between Python and R.

## Core Design Principles (Non-Negotiable)

- Explicit scene graph, no implicit state
- Deterministic intent and perceptually stable output
- Static rendering first, interaction optional
- GPU required, no legacy backends
- Identical feature surface for Python and R
- Narrow scope, long-term stability over features
- Errors must be early, explicit, and informative

**Primitive Rule (frozen):** If it can be expressed as a combination of existing primitives, it is not a new primitive.

Anything that violates these principles is out of scope.

## Coordinate System

Frustum uses the OpenGL convention:
- Right-handed coordinate system
- +X → right
- +Y → up
- −Z → forward (into the scene)
- +Z → toward the viewer

Camera looks down −Z in its local space.

## Required Feature Set

### 1. Scene Model

- Scene as a single immutable object
- Explicit camera definition:
  - eye (camera position)
  - target (look-at point)
  - up vector
  - projection type (orthographic, perspective)
  - near and far planes
  - fov_y (perspective) or view_height (orthographic)
- Explicit coordinate system
- Explicit scene bounds
- Schema version identifier (e.g. `frustum/scene/v1`)

No hidden defaults beyond documented, frozen ones.

### 2. Geometry Primitives

Five primitives (complete set):

**Points** — unordered 0D geometry
- positions, optional scalars, uniform size

**Lines** — piecewise linear 1D geometry
- ordered positions, optional scalars, uniform width

**Curves** — parametric/spline 1D geometry
- control points, curve type, explicit sampling
- evaluated to Lines before rendering

**Meshes** — 2D manifold in 3D (triangles only)
- positions, indices, optional normals, optional scalars

**Axes** — explicit reference geometry
- bounds, ticks, labels
- expanded to Lines before rendering

Out of scope (v1):
- text as 3D geometry
- volumetric rendering
- quads (all meshes are triangles)

### 3. Marching Cubes

Pipeline: volume → Mesh → render

- Input: regular 3D scalar volume
- Configurable voxel spacing and origin
- Single or multiple iso-levels
- Output: Mesh primitive (triangle mesh)
- Normals must be stable and well-defined
- Optional pre-smoothing
- Optional mesh decimation (bounded, explicit)

Marching cubes is a pre-processing step, not a renderer feature.

### 4. Materials and Color

- Shared colormap system compatible with Matplotlib semantics
- Scalar to color mapping
- Fixed color space (documented)
- Simple lighting model:
  - single directional light or headlight
  - Lambertian shading only

Out of scope:
- complex PBR
- shadows
- reflections
- transparency beyond very limited, explicit cases

### 5. Axes and Reference Geometry

- Optional axes as explicit geometry
- Tick marks and labels rendered deterministically
- No automatic layout magic

Axes are geometry, not UI.

### 6. Rendering Backend

- Rust + wgpu
- WGSL shaders
- Headless rendering support
- Render-to-image (PNG required)
- Fixed resolution and DPI

No browser dependency in the authoritative pipeline.

### 7. Determinism Guarantees

Frustum guarantees:
- Same scene, same topology, same camera produces same figure meaning
- No hidden state
- No order-dependent rendering unless explicitly documented

Frustum does not guarantee:
- bitwise identical pixels across GPUs

This distinction must be documented clearly.

### 8. Cross-Language Frontends

- Python and R frontends are feature-equivalent
- Both compile to the same scene schema
- No language-specific features
- No divergence in defaults

If a feature exists in one frontend, it must exist in the other or be removed.

### 9. Schema and Validation

- Canonical scene schema (TypeScript-defined)
- JSON schema derivation
- Validation before rendering
- Clear error messages

Invalid scenes must fail early.

### 10. Testing and CI

- Golden image tests with tolerance
- Cross-platform CI (Linux, macOS, Windows)
- Schema validation tests
- Marching cubes regression tests

### 11. Documentation

- Design rationale
- Explicit non-goals
- Reproducibility guarantees
- Minimal examples
- One complete end-to-end example per frontend

Documentation is part of the feature set.

## Explicit Non-Goals (Frozen)

Frustum will not:
- replace Matplotlib
- provide 2D plotting
- support dashboards or GUIs
- provide animation systems
- support legacy hardware or APIs
- chase performance benchmarks for marketing
- add features without design justification

## Success Criteria

Frustum is successful if:
- A scientist can generate a 3D figure reproducibly in Python or R
- The figure is acceptable in a paper without apology
- The API feels boring, explicit, and predictable
- The system can be maintained without architectural rewrites
