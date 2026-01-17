# Frustum — Document 009

## Extensions and Future Geometry Pipelines

### Purpose

This document enumerates explicitly supported future extensions to Frustum's geometry generation model.

These extensions are:
- Out of scope for v0.1
- Non-normative
- Semantically compatible with the frozen core (Features 001–008)

This document exists to:
- Show architectural foresight
- Prevent ad-hoc feature creep
- Clarify what could be added without breaking Frustum's philosophy

---

### Guiding rule for all extensions

**Any extension must reduce to existing primitives before reaching the renderer.**

If it does not produce:
- Points
- Lines
- Curves
- Meshes
- Axes

then it is not a Frustum extension.

---

### Extension 1 — Dual Contouring (Volume → Mesh)

**Description:**
Alternative isosurface extraction algorithm that:
- Preserves sharp features
- Operates on implicit surfaces
- Produces meshes

**Status:**
- Not included in v0.1
- Semantically compatible with Feature 002 (Meshes)

**Rationale for deferral:**
- Requires Hermite data
- Increases algorithmic complexity
- Less common in scientific scalar fields

**Compatibility:**
- ✓ Output: Mesh
- ✓ Deterministic with fixed rules
- ✓ Can coexist with Marching Cubes

---

### Extension 2 — Surface Nets

**Description:**
Voxel-based surface extraction producing:
- Quad-dominant meshes
- Simplified topology

**Status:**
- Out of scope
- Considered optional future generator

**Notes:**
- Would require triangulation before rendering
- Mostly relevant for voxel engines, not scientific viz

---

### Extension 3 — Signed Distance Field (SDF) Inputs

**Description:**
Accept explicit signed distance fields as volume input.

**Key point:**
Frustum does not define how SDFs are produced.

**Geometry generation:**
- SDF → isosurface extraction
- MC or Dual Contouring used internally

**Compatibility:**
- ✓ Deterministic if SDF is deterministic
- ✓ Reduces to Mesh

---

### Extension 4 — Multi-Isovalue Extraction

**Description:**
Extract multiple isosurfaces from a single volume:
- Multiple iso-values
- Ordered, explicit
- Produces multiple meshes

**Use cases:**
- Layered structures
- Threshold comparisons

**Compatibility:**
- ✓ Pure preprocessing
- ✓ No renderer changes required

---

### Extension 5 — Derived Scalar Fields

**Description:**
Generate new scalar fields from existing ones:
- Gradients
- Curvature
- Laplacian
- Local variance

**Purpose:**
Enhance analysis, not geometry.

**Flow:**
```
Volume → Derived scalar → Material mapping
```

**Compatibility:**
- ✓ Feeds directly into ScalarMappedMaterial
- ✓ No geometry mutation

---

### Extension 6 — GPU-Accelerated Geometry Generation

**Description:**
Implement geometry generators (e.g. Marching Cubes) using:
- Compute shaders
- GPU kernels

**Important clarification:**
This is an implementation optimization, not a semantic change.

**Compatibility:**
- ✓ Output identical Mesh primitives
- ✓ Semantics unchanged

---

### Extension 7 — Adaptive / Hierarchical Isosurfaces

**Description:**
Isosurface extraction on:
- Octrees
- Adaptive grids

**Status:**
- Explicitly out of scope for v0.1

**Reason for caution:**
- Introduces hierarchy
- Complicates determinism
- Harder to validate

**Compatibility:**
Only acceptable if:
- Final output is a flat Mesh
- Hierarchy is not exposed to the Scene

---

### Extension 8 — Neural Implicit Pipelines (Compatibility Only)

**Description:**
Neural models (e.g. SDF networks, NeRF-derived surfaces) that:
- Define implicit geometry
- Require surface extraction

**Frustum position:**

Frustum does not:
- Train models
- Evaluate networks
- Define learning semantics

Frustum can consume:
- Meshes produced by such pipelines

**Key insight:**
Even neural pipelines typically terminate in Marching Cubes.

---

### Extension 9 — Parametric Surface Generators

**Description:**
Parametric definitions:
- Bézier surfaces
- Spline patches
- Analytic surfaces

**Handling:**
- Evaluated explicitly
- Sampled deterministically
- Converted to Mesh

**Compatibility:**
- ✓ Deterministic sampling
- ✓ Explicit resolution
- ✓ Mesh output

---

### Explicit non-extensions

The following are **not** considered Frustum extensions:

| Pattern | Reason |
|---------|--------|
| Real-time LOD systems | Runtime mutation |
| Streaming geometry | Violates immutability |
| Procedural runtime mutation | Non-deterministic |
| ECS-style scene mutation | Rejected architecture |
| Implicit renderer-side geometry | Violates 007 contract |

These are incompatible with Frustum's goals.

---

### Architectural conclusion

Frustum's geometry layer is **closed but extensible**:

| Aspect | Status |
|--------|--------|
| Primitives | Closed (5 types) |
| Generators | Open (extensible) |
| Semantics | Stable (frozen) |

Marching Cubes is not a legacy artifact — it is the canonical boundary between volumetric data and explicit geometry.

---

## Flash at the Edges

### Guiding principle (anchor this)

**Flash is allowed only where it is reversible.**

If you can turn it off and the figure is still scientifically valid, it's allowed.
If turning it off breaks meaning, it doesn't belong.

Everything below follows that rule.

---

### Layer 1 — Geometry generators (extensions)

Already enumerated above (Dual Contouring, SDF, parametric surfaces).

Flashy but safe:
- Dual Contouring (sharp features)
- Multi-iso extraction with layered materials
- SDF-based generators
- Parametric surface generators

Why this works:
- Output is still Mesh
- Renderer doesn't care
- Core semantics untouched

---

### Layer 2 — Materials (not geometry)

This is where visual punch lives.

Post-v0.1 additions (non-normative):
- Smooth vs flat shading toggle
- Contour lines as material overlay
- Curvature-based coloring
- Edge highlighting
- Silhouette enhancement

All of these:
- Are material effects
- Don't alter topology
- Can be disabled cleanly

---

### Layer 3 — Lighting presets

Minimal lighting in v0.1 was correct.

Post-v0.1 named lighting rigs (non-normative):
- `scientific_flat`
- `studio_soft`
- `rim_highlight`
- `depth_emphasis`

Requirements:
- Explicit (no hidden defaults)
- Documented
- Opt-in

---

### Layer 4 — Camera choreography

No interaction ≠ boring.

Post-v0.1 additions (non-normative):
- Scripted camera paths
- Turntable renders
- Exploded views
- Multi-view grids

These are:
- Deterministic
- Declarative
- Publication-friendly

---

### Layer 5 — Presentation surfaces

The sleeper move: you don't just ship a renderer — you ship outputs that travel.

Post-v0.1 targets (non-normative):
- PNG (baseline, v0.1)
- SVG overlays for axes
- HTML/WebGPU viewer (same scene schema)
- Video turntables
- Interactive notebooks (read-only scene playback)

The same Scene rendered in:
- Rust + wgpu for papers
- WebGPU for demos

This is flashy — and totally aligned with the design.

---

### Anti-patterns (never do these)

These kill long-term credibility:

| Pattern | Problem |
|---------|---------|
| Auto camera fitting | Hidden state |
| Hidden lighting defaults | Non-reproducible |
| Adaptive LOD without user intent | Runtime mutation |
| "Magic" smoothing | Implicit data modification |
| Implicit color tricks | Non-explicit semantics |

These look cool for 10 minutes and cost you 10 years.

---

## Material Graphs (Node-Based Coloring)

### Design note (freeze this)

**Future material graphs are restricted, declarative DAGs for scalar-to-color mapping, not general shader graphs.**

---

### What node-based coloring actually is

Strip the hype. In modern engines, node graphs mean:

1. **Material graphs** — scalar → color → lighting → output
2. **Data-flow graphs** — derived values (gradients, curvature) → mapping
3. **Shader authoring UIs** — arbitrary GPU code with knobs

Only (1) and constrained (2) are compatible with Frustum. (3) is a hard no.

---

### Where it belongs

Material graphs belong **entirely inside the Materials layer**.

- Not geometry
- Not renderer logic
- Not Scene mutation

Reframe as: **Material Graphs = declarative scalar-to-appearance pipelines**

---

### The correct abstraction

A restricted DAG with:
- No loops
- No conditionals
- No side effects
- No texture sampling (v0.x)
- No time dependence

Think functional composition, not Blender shaders.

---

### Allowed node types

**Input nodes**
- `scalar` (from geometry)
- `normal` (for lighting modulation)
- `constant` (literal value)

**Math nodes**
- `add`, `multiply`, `subtract`, `divide`
- `normalize`
- `clamp`
- `smoothstep`
- `power`
- `abs`

**Mapping nodes**
- `remap_range`
- `colormap_lookup`
- `quantize` / banding

**Output node**
- `rgba` (final color)

Every node: **pure, deterministic, explicitly typed**.

---

### Explicitly forbidden

These are renderer poison:

| Pattern | Reason |
|---------|--------|
| Branching on screen space | Non-reproducible |
| Noise / randomness | Non-deterministic |
| Time-based nodes | Animation scope |
| Texture lookups | Complexity, v0.x |
| Camera-dependent logic | View coupling |
| Backend-specific nodes | Portability |

**Rule:** If a node can't be expressed identically in WGSL and CPU reference code, it doesn't exist.

---

### Integration with frozen model

Current state (Feature 004):
- `SolidMaterial`
- `ScalarMappedMaterial`

Natural evolution (post-v0.1):
- `MaterialGraph` — references a graph definition, produces RGBA, binds via `material_id`

From the renderer's perspective: still "apply material → get color". Nothing else changes.

---

### The determinism win

Node-based coloring is **more deterministic** than ad-hoc shader code if you:
- Restrict the node set
- Freeze evaluation order
- Define exact math semantics
- Forbid backend shortcuts

This gives you:
- Reproducible styling
- Inspectable appearance logic
- Portable materials (Rust ↔ WebGPU ↔ Python)

---

### The sane constraint

**A material graph must be serializable, evaluable on CPU, and testable without a GPU.**

If that's true:
- You can unit-test materials
- You can generate reference images
- You can port to WebGPU later
- Reviewers will trust it

---

### What this unlocks (the flash)

- Curvature-based highlights
- Contour overlays
- Banded scientific colormaps
- Edge emphasis
- Data-driven stylization

This is the good kind of flashy: looks modern, still honest, still reproducible.

---

### Status

- This document is **informational**
- No implementation is implied
- No commitments are made

---

### Document classification

| Property | Value |
|----------|-------|
| Type | Non-normative |
| Scope | Future planning |
| Binding | None |
| Version | v0.1 |
