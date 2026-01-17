# Frustum — Feature Prompt 002

## Geometry Primitives

### Purpose

Define the complete set of geometry primitives for Frustum.
This feature specifies `AnyRenderable` from Feature 001.

Primitives are geometric atoms. Everything else is composition.

---

### Core Design Rule (frozen)

**If it can be expressed as a combination of existing primitives, it is not a new primitive.**

This rule prevents primitive explosion and keeps the renderer tractable.

---

### Scope

This feature defines:
- Points
- Lines
- Curves
- Meshes
- Axes

This feature does not include:
- scatter plots (composition of Points)
- surface plots (composition of Meshes)
- contour plots (composition of Lines)
- quiver plots (composition of Lines)
- volume rendering
- text geometry
- glyphs

---

### Design requirements (non-negotiable)

- Primitives are minimal and orthogonal
- No implicit triangulation or tessellation
- All sampling/resolution is explicit
- Primitives are immutable once validated
- Primitives serialize to the scene schema
- No rendering logic in primitive definitions

**Appearance rule (frozen):** Geometry primitives must not define visual appearance directly. All color and shading is expressed exclusively via material references (see Feature 004).

---

### Primitives

#### 1. Points

Unordered 0-dimensional geometry.

Required fields:
- `positions`: vec3[] — vertex positions

Optional fields:
- `id`: string — stable identifier for reference/debugging
- `scalars`: float[] — per-point scalar for colormap
- `size`: float — uniform point size (pixels)
- `material_id`: string — reference to material (required for rendering)

Constraints:
- positions.length must be divisible by 3
- If scalars present, scalars.length == positions.length / 3
- size > 0 if specified

Covers:
- scatter plots
- sampled point clouds
- markers

---

#### 2. Lines

Piecewise linear 1-dimensional geometry.

Required fields:
- `positions`: vec3[] — ordered vertex positions

Optional fields:
- `id`: string — stable identifier for reference/debugging
- `scalars`: float[] — per-vertex scalar for colormap
- `width`: float — uniform line width (pixels)
- `material_id`: string — reference to material (required for rendering)

Constraints:
- positions.length must be divisible by 3
- positions.length >= 6 (at least 2 points)
- If scalars present, scalars.length == positions.length / 3
- width > 0 if specified

Covers:
- trajectories
- polylines
- graph edges
- contours (after projection)

Note: No distinction between "polyline" and "line plot". Both are Lines.

---

#### 3. Curves

Parametric or spline-defined 1-dimensional geometry.

Required fields:
- `curve_type`: enum { `cubic_bezier`, `catmull_rom`, `b_spline` }
- `control_points`: vec3[] — control points or knots
- `segments`: int — number of line segments for evaluation

Optional fields:
- `id`: string — stable identifier for reference/debugging
- `scalars`: float[] — per-evaluated-vertex scalar
- `width`: float — uniform line width (pixels)
- `material_id`: string — reference to material (required for rendering)

Constraints:
- Control point count must match curve type requirements
- segments >= 1
- width > 0 if specified

Rendering behavior:
- Curves are evaluated to Lines before rendering
- Sampling is explicit and deterministic
- The renderer sees only the evaluated Lines

Covers:
- smooth trajectories
- spline interpolations
- parametric curves

---

#### 4. Meshes

2-dimensional manifold geometry embedded in 3D.

Required fields:
- `positions`: vec3[] — vertex positions
- `indices`: uint[] — triangle indices (3 per triangle)

Optional fields:
- `id`: string — stable identifier for reference/debugging
- `normals`: vec3[] — per-vertex normals
- `scalars`: float[] — per-vertex scalar for colormap
- `material_id`: string — reference to material (required for rendering)

Constraints:
- positions.length must be divisible by 3
- indices.length must be divisible by 3
- All indices must be valid (< vertex count)
- If normals present, normals.length == positions.length
- If scalars present, scalars.length == positions.length / 3

Covers:
- surfaces
- isosurfaces (marching cubes output)
- triangulated parametric surfaces

Note: No quads. No implicit triangulation. All meshes are explicit triangle meshes.

---

#### 5. Axes

Axes are a first-class primitive providing explicit reference geometry.

**Deferred to Feature 005:** The complete structure, tick logic, label placement, and generation rules for Axes are defined exclusively in Feature 005.

Summary:
- Axes are optional
- Axes are explicit, not UI
- Axes expand to Lines before rendering
- Axes are geometry; they receive no special treatment in the renderer

---

### AnyRenderable (tagged union)

The `objects` field in Scene contains a list of:

```
AnyRenderable = Points | Lines | Curves | Meshes | Axes
```

Each variant is tagged with a `type` field in JSON:
- `"points"`
- `"lines"`
- `"curves"`
- `"mesh"`
- `"axes"`

---

### Validation rules

A primitive is invalid if:
- Any required field is missing
- Any position/normal contains NaN or Inf
- Any index is out of bounds
- Array lengths violate constraints
- `material_id` references a non-existent material
- Scalar-mapped material is referenced but primitive lacks `scalars`

Validation errors must:
- name the primitive index in the scene
- name the offending field
- explain why it is invalid
- fail fast

---

### Serialization

All primitives serialize to JSON as part of the scene.

Position arrays are flattened: `[x0, y0, z0, x1, y1, z1, ...]`

Binary geometry buffers are out of scope for v0.1.

---

### Cross-language parity

Python and R frontends must:
- expose identical primitive constructors
- use identical defaults
- produce semantically equivalent JSON

---

### Success criteria

This feature is complete when:
- All five primitives can be created in Python and R
- Primitives serialize correctly to scene JSON
- Validation catches malformed primitives reliably
- Curves evaluate to Lines deterministically
- Axes expand to Lines deterministically
- The schema can be extended to `frustum/scene/v1` with primitives

---

### Explicit non-goals

This feature does not include:
- convenience constructors for scatter/surface/contour
- automatic normal generation
- automatic triangulation
- text rendering
- per-segment scalars for Lines (deferred)
- instanced geometry
- volumetric primitives

---

### Rationale

Primitives are the atoms of the scene graph.
If primitives are bloated, the renderer becomes complex.
If primitives are too sparse, users must reinvent composition.

Five primitives is the minimum viable set that covers:
- point clouds
- trajectories
- smooth curves
- surfaces and isosurfaces
- reference geometry

This set is complete for scientific figures without being complete for games.

---

### Next

Feature Prompt 003: Marching Cubes (volume → mesh → render)
