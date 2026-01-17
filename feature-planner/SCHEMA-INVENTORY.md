# Frustum Schema Field Inventory

**Status:** COMPLETE — Features 001–008 inventoried, all issues resolved

---

## Feature 001 — Scene & Camera

### Scene

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `version` | string | yes | 001 | 001: unsupported version |
| `camera` | Camera | yes | 001 | 001: all camera rules |
| `world_bounds` | AABB | yes | 001 | 001: authoritative extent |
| `objects` | list[AnyRenderable] | yes | 001 | 002: per-primitive rules |
| `materials` | list[Material] | yes | 001 | 004: per-material rules |
| `light` | Light | no | 006 | 006: lighting rules |

### Camera

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `eye` | vec3 | yes | 001 | 001: finite, ≠ target |
| `target` | vec3 | yes | 001 | 001: finite, ≠ eye |
| `up` | vec3 | yes (default: 0,1,0) | 001 | 001: not collinear with view |
| `projection` | enum | yes | 001 | 001: perspective \| orthographic |
| `near` | float | yes (default: 0.01) | 001 | 001: positive, < far |
| `far` | float | yes (default: 1000.0) | 001 | 001: positive, > near |
| `fov_y` | float | perspective only | 001 | 001: positive, degrees |
| `view_height` | float | orthographic only | 001 | 001: positive, world units |

---

## Feature 002 — Geometry Primitives

### Common (all primitives)

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `type` | string | yes | 002 | 002: discriminator tag |
| `id` | string | no | 002 | 002: stable identifier for reference/debugging |
| `material_id` | string | no* | 002 | 002: ref exists in Scene.materials |

*Required for rendering, optional in schema

### Points

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `positions` | vec3[] | yes | 002 | 002: length % 3 == 0, finite |
| `scalars` | float[] | no | 002 | 002: length == vertex count |
| `size` | float | no | 002 | 002: > 0 if specified |

### Lines

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `positions` | vec3[] | yes | 002 | 002: length ≥ 6, % 3 == 0, finite |
| `scalars` | float[] | no | 002 | 002: length == vertex count |
| `width` | float | no | 002 | 002: > 0 if specified |

### Curves

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `curve_type` | enum | yes | 002 | 002: cubic_bezier \| catmull_rom \| b_spline |
| `control_points` | vec3[] | yes | 002 | 002: count matches curve type |
| `segments` | int | yes | 002 | 002: ≥ 1 |
| `scalars` | float[] | no | 002 | 002: length == evaluated vertex count |
| `width` | float | no | 002 | 002: > 0 if specified |

### Meshes

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `positions` | vec3[] | yes | 002 | 002: length % 3 == 0, finite |
| `indices` | uint[] | yes | 002 | 002: length % 3 == 0, valid refs |
| `normals` | vec3[] | no | 002 | 002: length == positions.length |
| `scalars` | float[] | no | 002 | 002: length == vertex count |

### Axes

Deferred to Feature 005 (see AxisBundle below).

---

## Feature 003 — Marching Cubes

### Volume (input)

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `values` | float[][][]| yes | 003 | 003: finite, dims ≥ 2 |
| `spacing` | vec3 | yes | 003 | 003: strictly positive |
| `origin` | vec3 | yes | 003 | 003: finite |
| `iso_value` | float | yes | 003 | 003: finite |

### Optional modifiers

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `smoothing.kernel_size` | int | no | 003 | 003: explicit if smoothing |
| `smoothing.sigma` | float | no | 003 | 003: explicit if smoothing |
| `decimation.target` | int/float | no | 003 | 003: explicit if decimation |

### Output

Mesh primitive (fields defined in 002). No `material_id` — user must assign.

---

## Feature 004 — Materials

### Material (base)

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `id` | string | yes | 004 | 004: unique in Scene.materials |
| `type` | enum | yes | 004 | 004: solid \| scalar_mapped |

### SolidMaterial

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `color` | RGB(A) | yes | 004 | 004: values in [0, 1] |

### ScalarMappedMaterial

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `colormap` | string | yes | 004 | 004: known identifier |
| `range` | (min, max) | yes | 004 | 004: finite, min < max |
| `clamp` | boolean | no (default: true) | 004 | 004: — |
| `missing_color` | RGB(A) | no | 004 | 004: values in [0, 1] |

---

## Feature 005 — Axes

### AxisBundle

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `id` | string | yes | 005 | 005: unique |
| `bounds` | AABB | yes | 005 | 005: non-degenerate, ⊆ world_bounds |
| `axes` | list[string] | yes | 005 | 005: non-empty subset of [x,y,z] |
| `material_id` | string | yes | 005 | 005: exists, is SolidMaterial |
| `ticks` | TickSpec | no | 005 | 005: tick rules |
| `labels` | LabelSpec | no | 005 | 005: label rules |

### TickSpec

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `mode` | enum | yes | 005 | 005: fixed \| auto |
| `values` | float[] | if fixed | 005 | 005: within bounds |
| `count` | int | if auto | 005 | 005: ≥ 1 |

### LabelSpec

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `show` | boolean | yes | 005 | 005: — |
| `format` | string | no | 005 | 005: valid format string |
| `offset` | vec3 | yes | 005 | 005: finite |

---

## Feature 006 — Lighting

### Light

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `direction` | vec3 | yes | 006 | 006: finite, normalized |
| `intensity` | float | yes | 006 | 006: finite, ≥ 0 |
| `enabled` | boolean | no (default: true) | 006 | 006: — |

---

## Feature 007 — Renderer Contract

### RenderConfig

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `width` | int | yes | 007 | 007: > 0 |
| `height` | int | yes | 007 | 007: > 0 |
| `background_color` | RGBA | no (default: white) | 007 | 007: values in [0, 1] |
| `pixel_ratio` | float | no (default: 1.0) | 007 | 007: > 0 |

---

## Feature 008 — Text Rendering

### ExpandedLabel (internal)

| Field | Type | Required | Owner | Validation |
|-------|------|----------|-------|------------|
| `text` | string | yes | 008 | 008: non-empty, ASCII only |
| `position` | vec3 | yes | 008 | 008: finite |
| `size` | float | yes | 008 | 008: > 0 |
| `material_id` | string | yes | 008 | 008: exists in Scene.materials |

Note: ExpandedLabel is generated during axes expansion, not user-specified.

---

## Issues Found

### Issue 1: Missing `id` field on primitives (002) — RESOLVED

**Expected:** Primitives should have optional `id` field for reference/debugging.
**Actual:** Feature 002 did not define `id` for Points, Lines, Curves, Meshes.

**Resolution:** Added optional `id: string` to all primitives in 002. ✓

### Issue 2: LabelSpec.show and LabelSpec.offset requiredness — RESOLVED

**Expected:** LabelSpec fields should have clear required/optional status.
**Actual:** 005 listed `show` and `offset` as "Fields" without explicit required marker.

**Resolution:** Clarified in 005 — `show` required, `offset` required (with documented default). ✓

---

## Cross-Reference Verification

| Reference | From | To | Status |
|-----------|------|----|--------|
| Scene.objects → AnyRenderable | 001 | 002 | ✓ |
| Scene.materials → Material | 001 | 004 | ✓ |
| Scene.light → Light | 001 | 006 | ✓ |
| primitive.material_id → Material.id | 002 | 004 | ✓ |
| AxisBundle.material_id → SolidMaterial | 005 | 004 | ✓ |
| Marching cubes output → Mesh | 003 | 002 | ✓ |
| Axes → Lines (expansion) | 005 | 002 | ✓ |
| AxisBundle.bounds ⊆ Scene.world_bounds | 005 | 001 | ✓ |
| Renderer consumes Scene | 007 | 001 | ✓ |
| Renderer consumes Light | 007 | 006 | ✓ |
| ExpandedLabel.material_id → Material | 008 | 004 | ✓ |
| Axes labels → ExpandedLabel | 005 | 008 | ✓ |

---

## Validation Rule Ownership

| Rule | Owner | Location |
|------|-------|----------|
| Scene-level validation | 001 | 001 validation section |
| Camera validation | 001 | 001 validation section |
| Primitive validation | 002 | 002 validation section |
| Material validation | 004 | 004 validation section |
| Material-primitive compatibility | 002+004 | Both (cross-validated) |
| Axes validation | 005 | 005 validation section |
| Marching cubes validation | 003 | 003 validation section |
| Light validation | 006 | 006 validation section |
| RenderConfig validation | 007 | 007 validation section |
| Label validation | 008 | 008 validation section |

---

## Completion Status

- [x] Every field has exactly one owner
- [x] Every field has a validation rule location
- [x] No field is defined in multiple features
- [x] All cross-references resolve
- [x] Issue 1: Add `id` to primitives (002)
- [x] Issue 2: Clarify LabelSpec requiredness (005)

**Frustum v0.1 — FROZEN (001–008)**

---

## Non-normative Documents

| Document | Description |
|----------|-------------|
| 009-extensions.md | Future geometry pipelines (informational, no commitments) |
| 010-implementation-notes.md | Pre-implementation review findings (guidance, not spec) |
