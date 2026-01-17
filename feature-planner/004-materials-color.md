# Frustum — Feature Prompt 004

## Materials and Scalar → Color Mapping

### Purpose

Define how visual appearance is attached to geometry primitives in Frustum, without altering geometry, topology, or scene semantics.

Materials control how things look, never what they are.

---

### Scope

This feature defines:
- Material objects
- Scalar → color mapping
- Colormap semantics
- Lighting interaction at a high level

This feature does not include:
- Geometry generation
- Lighting models beyond a minimal contract
- Transparency pipelines
- Texturing
- Styling systems or themes

---

### Core design principles

- Materials are declarative
- Materials never modify geometry
- Scalar mapping is explicit and reproducible
- Defaults are minimal and frozen
- Appearance must not affect topology or ordering

**No implicit materials (frozen):** Geometry primitives must not define visual appearance directly (no inline `color` field). All appearance is expressed exclusively via material references. `SolidMaterial` is the canonical way to express uniform color. No auto-generation of anonymous materials. Convenience belongs in frontend helpers, not the schema.

---

### Material model

A Material is a standalone object that can be referenced by renderable primitives.

Required fields:
- `id`: string (stable identifier)
- `type`: enum { `solid`, `scalar_mapped` }

---

### Material types

#### 1. SolidMaterial

Applies a uniform color.

Fields:
- `color`: RGB or RGBA (floats in [0, 1])

Constraints:
- Alpha allowed but transparency behavior is restricted (see below)

---

#### 2. ScalarMappedMaterial

Maps scalar values to colors via a colormap.

Required fields:
- `colormap`: string identifier
- `range`: tuple (min, max)

Optional fields:
- `clamp`: boolean (default: true)
- `missing_color`: RGB(A) for NaN or missing scalars

Constraints:
- Scalar values must be supplied by the geometry
- Range must be finite and min < max

---

### Colormap system

#### Colormap identifiers

- Colormaps are referenced by name
- Initial set mirrors a stable subset of Matplotlib colormaps:
  - `viridis`
  - `plasma`
  - `inferno`
  - `magma`
  - `cividis`

Colormaps must be:
- continuous
- perceptually uniform
- fixed and versioned

No user-defined colormaps in v0.1.

---

### Scalar normalization

Scalar mapping is defined as:

```
t = (value - min) / (max - min)
```

Then:
- if `clamp = true`, t is clamped to [0, 1]
- if `clamp = false`, values outside range are invalid

This behavior must be documented and consistent.

---

### Binding materials to geometry

Renderable primitives reference materials by `id`.

Rules:
- A primitive may reference one material only
- Material lookup happens at validation time
- Missing material references are errors

No per-vertex material switching.

---

### Lighting interaction (minimal contract)

Materials must define how they interact with lighting, but lighting itself is not defined here.

For v0.1:
- Materials are assumed to be Lambertian
- No specular terms
- No shadows
- No reflections

This is a contract, not an implementation detail.

---

### Transparency rules (important constraint)

Transparency is explicitly limited in v0.1.

Rules:
- Alpha < 1.0 is allowed
- Order-dependent blending is not guaranteed
- Transparent materials may render approximately correct
- This limitation must be documented clearly

No sorting heuristics are required or promised.

---

### Validation rules

Materials are invalid if:
- Color values are outside [0, 1]
- Scalar-mapped material is used with geometry lacking scalars
- Colormap identifier is unknown
- Range is invalid or non-finite

Validation must:
- identify material id
- identify consuming geometry (if applicable)
- fail early

---

### Serialization and schema

- Materials are serialized as part of the Scene
- Materials live in a dedicated `Scene.materials` collection
- Geometry references materials by `id`

Materials are immutable once validated.

---

### Cross-language parity

Python and R frontends must:
- expose identical material types
- expose identical colormap identifiers
- use identical defaults
- serialize to equivalent canonical JSON

No frontend-specific color shortcuts.

---

### Explicit non-goals

This feature does not include:
- Textures
- Image-based materials
- Pattern fills
- Gradients beyond scalar mapping
- Per-face or per-edge materials
- Theme systems

---

### Success criteria

This feature is complete when:
- Geometry can reference solid and scalar-mapped materials
- Scalar mapping is visually consistent across platforms
- Invalid material usage fails clearly
- Python and R produce identical material bindings
- Appearance changes do not affect geometry or determinism

---

### Rationale

Most plotting systems entangle geometry and appearance early.

Frustum enforces a clean separation:

**Geometry defines what exists**
**Materials define how it looks**

This keeps the rendering pipeline simple, testable, and stable.

---

### Forward references

- Feature 005: Axes as explicit geometry
- Feature 006: Lighting model (minimal)
- Feature 007: Rendering pipeline (consumes geometry + materials)
