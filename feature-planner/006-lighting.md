# Frustum — Feature Prompt 006

## Lighting (Minimal, Deterministic)

### Purpose

Define the minimal lighting model supported by Frustum v0.1, constrained by Feature 007's renderer contract.

Lighting affects appearance only, never geometry, topology, or scene semantics.

---

### Scope

This feature defines:
- Lighting model
- Lighting parameters
- Where lighting configuration lives

This feature does not define:
- Advanced shading
- Shadows
- Transparency interactions
- PBR
- Multiple light types

---

### Design constraints (from 007)

- Lighting must be deterministic
- Lighting must not modify geometry
- Lighting must not introduce hidden defaults
- Lighting must be explicit

---

### Lighting model (v0.1)

**Model:** Single-light Lambertian shading

**Components:**
- Diffuse term only
- No specular
- No ambient occlusion
- No shadows

**Formula (conceptual):**

```
color = base_color * max(dot(normal, light_dir), 0)
```

**Normals:**
- Required for Mesh shading
- Ignored for Points and Lines (rendered unlit or constant-shaded)

---

### Lighting configuration location

**Decision:** Lighting lives in Scene, not RenderConfig.

**Rationale:**
- Lighting affects appearance semantics
- Lighting must be reproducible across renderers
- RenderConfig must not affect "what the figure means"

---

### Scene extension

Scene gains an optional field:

```
Scene {
    version: string
    camera: Camera
    world_bounds: AABB
    objects: list[AnyRenderable]
    materials: list[Material]
    light: Light (optional)        // NEW
}
```

---

### Light object

#### Light

Required fields:
- `direction`: vec3 — light direction in world space (normalized)
- `intensity`: float — light intensity (≥ 0)

Optional fields:
- `enabled`: boolean — whether lighting is applied (default: true)

Constraints:
- Direction must be finite
- Direction must be normalized (length ≈ 1)
- Intensity must be finite and ≥ 0
- No positional lights in v0.1

---

### Defaults (frozen)

**No-light rule (frozen):** If no Light is specified, no lighting is applied. Colors render flat (base material color only).

No implicit "headlight". No auto-generated lighting.

---

### Shading behavior by primitive type

| Primitive | Lighting behavior |
|-----------|-------------------|
| Mesh | Lambertian shading using vertex normals |
| Points | Unlit (flat material color) |
| Lines | Unlit (flat material color) |
| Curves | Unlit (flat material color, after expansion) |
| Axes | Unlit (flat material color, after expansion) |

Only Meshes with normals receive shading.

---

### Validation rules

Light is invalid if:
- Direction contains NaN or Inf
- Direction is zero-length
- Direction is not normalized (tolerance: length in [0.99, 1.01])
- Intensity < 0
- Intensity is NaN or Inf

Validation errors must:
- Name the offending field
- Explain why it is invalid
- Fail during Scene validation

---

### Determinism guarantees

Given:
- Same geometry
- Same normals
- Same light direction and intensity

Lighting output must be:
- Perceptually stable
- Independent of render order
- Independent of resolution

---

### Serialization

Light serializes as part of the Scene:

```json
{
    "light": {
        "direction": [0.0, 0.0, -1.0],
        "intensity": 1.0,
        "enabled": true
    }
}
```

If omitted, no lighting is applied.

---

### Cross-language parity

Python and R frontends must:
- Expose identical Light constructors
- Use identical defaults (no light if unspecified)
- Produce semantically equivalent JSON

---

### Explicit non-goals

- Multiple lights
- Ambient light
- Specular highlights
- Camera-relative lights ("headlight")
- Auto-generated lights
- Shadow mapping
- Soft shadows
- Environment maps

---

### Success criteria

Feature 006 is complete when:
- Lighting can be specified explicitly
- Absence of lighting produces flat colors
- Lighting never alters geometry
- Lighting behavior is fully predictable
- Meshes shade correctly with Lambertian model
- Non-mesh primitives render unlit

---

### Rationale

Scientific visualization requires predictable lighting.

Most systems default to complex lighting that:
- Varies across renders
- Hides data features
- Creates confusion

Frustum's minimal model:
- Is explicit
- Is deterministic
- Is sufficient for scientific figures
- Can be extended later without breaking compatibility

---

### Forward references

- Feature 007: Renderer contract (consumes lighting)
- Feature 008: Text rendering (labels are unlit)
