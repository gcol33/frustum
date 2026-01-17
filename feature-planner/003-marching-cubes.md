# Frustum — Feature Prompt 003

## Marching Cubes (Volume → Mesh Generator)

### Purpose

Define marching cubes as a deterministic geometry generation step that converts a regular 3D scalar volume into a Mesh primitive as defined in Feature 002.

Marching cubes is not a rendering feature.
It is a pure preprocessing algorithm.

---

### Scope

This feature defines:
- Volume input specification
- Isosurface extraction via marching cubes
- Mesh generation guarantees
- Validation and determinism constraints

This feature does not include:
- Volume rendering
- Interactive iso-level adjustment
- GPU-based marching cubes
- Rendering, lighting, or shading

---

### Design principles (critical)

- Marching cubes is pure and deterministic
- Same input → same mesh topology
- Output is always a Mesh primitive
- No implicit smoothing, decimation, or filtering
- No hidden heuristics

All non-trivial behavior must be explicit.

---

### Input specification

#### Volume

A volume is a regular 3D scalar field.

Required fields:
- `values`: 3D array of float (Nx × Ny × Nz)
- `spacing`: vec3 (dx, dy, dz)
- `origin`: vec3 (world-space location of voxel [0,0,0])

Constraints:
- All values must be finite
- Dimensions must be ≥ 2 in each axis
- Spacing must be strictly positive

No implicit unit assumptions.

---

#### Iso-level

- `iso_value`: float

Defines the scalar threshold for surface extraction.

---

### Mathematical preconditions

Marching cubes assumes:
- The scalar field is **continuous** between samples
- Linear interpolation between voxels is meaningful
- The field is **band-limited** relative to the sampling grid (Nyquist criterion)

Violations produce:
- Jagged surfaces (undersampling)
- Topology artifacts (aliasing)
- "False" features (high-frequency content folded into low frequencies)

These are sampling theory consequences, not algorithmic bugs.

**Categorical volumes** (labels, masks, thresholded data) violate the continuity assumption. They must be rejected unless the user explicitly opts in via a flag acknowledging the mathematical mismatch.

---

### Algorithmic contract

Marching cubes must:
- Use a fixed lookup table
- Use a fixed edge interpolation rule
- Produce triangle-only meshes
- Produce consistent vertex ordering
- Avoid topology changes for identical inputs

#### Disambiguation rule (frozen)

Classic MC has topological ambiguity in saddle-point configurations. Different disambiguation rules can change topology, create holes, or flip connectivity — even with identical scalar fields. This is discrete topology ambiguity, not floating-point noise.

**Frustum uses the asymptotic decider** for ambiguous face cases. The lookup table is part of the algorithm's mathematical definition and must not vary.

Rationale: The asymptotic decider resolves face ambiguity by evaluating the saddle point of the bilinear interpolant, producing topologically consistent results.

#### Vertex interpolation (frozen)

Edge vertices are computed as:

```
t = (iso - v0) / (v1 - v0)
p = p0 + t * (p1 - p0)
```

Where:
- `v0`, `v1` are scalar values at edge endpoints
- `p0`, `p1` are world-space positions of edge endpoints
- `t` is the interpolation parameter

This formula is linear and must not be "optimized" or reordered. Small differences in `v0`, `v1` or evaluation order can shift vertices slightly — this is expected and acceptable instability, bounded by the voxel spacing.

Interpolation is always performed in **world space** (after applying origin + spacing).

---

### Output specification

The output is a Mesh primitive with:
- `positions`: vec3[]
- `indices`: triangle indices
- `normals`: vec3[] (required)
- `scalars`: per-vertex scalar value (optional, original volume value at vertex)

Note: The output Mesh has no `material_id`. The user must assign a material before the mesh can be rendered (see Feature 004).

#### Normal computation (frozen)

Normals encode differential information. The choice of computation method is a mathematical model decision, not just shading.

**Frustum uses gradient-based normals** computed from the volume field:

```
n = -normalize(gradient(volume, vertex_position))
```

Where the gradient is estimated via central differences on the volume grid, then interpolated to the vertex position.

Rationale:
- Gradient normals capture the underlying field's differential structure
- They produce smoother shading than face normals on coarse grids
- They are independent of mesh triangulation

Face normals (triangle cross-products) and averaged face normals are explicitly **not used**.

Normals are part of the generator's output, not derived by the renderer.

---

### World-space mapping

Voxel coordinates map to world space as:

```
world_position = origin + spacing * voxel_coordinate
```

Interpolation must be linear in world space.

---

### Optional explicit modifiers (v0.1 allowed, but optional)

If included, they must be explicit parameters:

#### 1. Pre-smoothing
- Simple Gaussian smoothing
- Kernel size and sigma explicit
- Applied before isosurface extraction

Pre-smoothing is a **signal-processing step** that attenuates high-frequency content. It does not "fix" aliased geometry — it changes the input field. Users must understand this distinction.

#### 2. Mesh decimation
- Deterministic algorithm only
- Target triangle count or reduction ratio explicit
- Must preserve topology where possible

If not explicitly enabled, no smoothing or decimation occurs.

---

### Validation rules

Marching cubes must fail if:
- Volume contains NaN or Inf
- Iso-value is outside volume value range (warning, not error)
- Output mesh is empty (warning, not error)

Failures must:
- explain the cause
- not produce partial geometry

---

### Determinism guarantees

Frustum guarantees:
- Identical volume + iso-value + parameters → identical mesh topology
- Vertex order is stable
- Triangle winding is consistent

Frustum does not guarantee:
- identical floating-point coordinates across hardware

Topology stability is the guarantee, not bitwise identity.

---

### Cross-language parity

- Marching cubes is implemented once (core, Rust)
- Python and R frontends call the same implementation
- No reimplementation in frontends
- No language-specific behavior

---

### Explicit non-goals

This feature does not include:
- Volume slicing
- Multi-material volumes
- Adaptive iso-surfaces
- GPU compute pipelines
- Time-varying volumes
- Interactive sliders

---

### Testing requirements

#### Topology regression tests

At least one test must use a **known ambiguous scalar field** that exercises saddle-point disambiguation. This test verifies:
- Topology is stable across runs
- The asymptotic decider produces the expected connectivity
- No holes or flipped faces in the output mesh

#### Geometric invariant tests

Geometric tests must use **tolerance-based comparisons**, not exact coordinate checks:
- Vertex positions within voxel-spacing tolerance
- Normals within angular tolerance (e.g. 1°)
- Mesh bounding box within tolerance of expected

Exact floating-point equality is not a valid test criterion.

---

### Success criteria

This feature is complete when:
- A volume can be converted into a Mesh
- The mesh validates under Feature 002
- The same input always produces the same topology
- Topology regression test passes for ambiguous cases
- Geometric invariant tests pass with documented tolerances
- Python and R produce identical meshes
- The mesh renders correctly via the existing pipeline

---

### Rationale

Marching cubes is widely used, but often embedded inside visualization stacks in opaque ways.

Frustum treats it as:

**pure geometry generation**

This separation:
- simplifies reasoning
- improves reproducibility
- avoids renderer complexity
- keeps scope contained

#### Why Marching Cubes (not alternatives)

| Algorithm | Assumption shift | Trade-off |
|-----------|------------------|-----------|
| Dual Contouring | Uses gradients, preserves sharp features | Requires Hermite data, more complex |
| Surface Nets | Trades accuracy for simplicity | Less precise vertex placement |
| Adaptive MC | Assumes hierarchical sampling | Requires octree, non-regular grids |
| Neural implicits | Replaces explicit field with learned function | Non-deterministic, opaque |

None of these dominate MC mathematically — they optimize for different priors.

MC aligns with Frustum's requirements:
- **Regular grids** (explicit sampling)
- **Deterministic reconstruction** (reproducible)
- **Explicit field values** (no learned components)

This is a mathematical alignment, not conservatism.

#### Separation of generation and consumption

Frustum separates geometry generation from geometry consumption. Mathematically, this means:
- Reconstruction errors live in the generator
- Rendering does not amplify or reinterpret them

Most visualization stacks blur this line. Frustum's separation is a deliberate architectural stance with mathematical implications

---

### Forward references

- Feature 004: Materials and scalar → color mapping
- Feature 005: Axes as explicit geometry
- Feature 006: Rendering pipeline (consumes Mesh, does not modify it)
