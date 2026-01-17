# Frustum — Feature Prompt 005

## Axes as Explicit Geometry

### Purpose

Define axes as explicit, deterministic geometry that provides spatial reference without introducing UI state, implicit layout, or rendering-side heuristics.

Axes in Frustum are not widgets.
They are geometry bundles generated in world space.

**This feature is the authoritative definition of the Axes primitive referenced in Feature 002.**

---

### Scope

This feature defines:
- Axis geometry specification
- Tick generation rules
- Label placement rules (geometry-level, not text rendering)
- Axis bounds and orientation

This feature does not include:
- Automatic layout
- Viewport-relative placement
- Interactive axes
- Text rendering implementation (glyphs)
- 2D plotting semantics

---

### Core design principles

- Axes are optional
- Axes are explicit
- Axes are generated deterministically
- Axes are ordinary geometry
- Axes do not depend on screen resolution or backend

No hidden coupling to the camera.

---

### Axis object model

Axes are represented as a single structured object that expands into geometry.

#### AxisBundle

Required fields:
- `id`: string
- `bounds`: axis-aligned bounding box (min, max) — typically matches `Scene.world_bounds`
- `axes`: list of enabled axes (any subset of `["x", "y", "z"]`)
- `material_id`: material for axis lines and ticks (required)

Optional fields:
- `ticks`: TickSpec
- `labels`: LabelSpec

**Material rule (frozen):** Axes must reference a material via `material_id`. Axes have no implicit colors or styling. Ticks and labels inherit the axis material — no separate tick or label materials.

---

### Axis geometry definition

For each enabled axis:
- One main axis line
- Zero or more tick marks
- Zero or more tick labels (as placeholders)

All geometry is generated in world coordinates.

---

### Tick specification

#### TickSpec

Required fields:
- `mode`: enum { `fixed`, `auto` }

If `fixed`:
- `values`: list of floats (world coordinates)

If `auto`:
- `count`: integer (approximate target)

Constraints:
- Tick values must lie within axis bounds
- Auto ticks must be generated deterministically

No "nice number" heuristics that depend on floating-point quirks.

---

### Label specification (geometry-level)

Labels are semantic placeholders, not rendered text.

#### LabelSpec

Required fields:
- `show`: boolean — whether to generate label placeholders
- `offset`: vec3 — world-space offset from tick position (default: [0.1, 0, 0])

Optional fields:
- `format`: format string (e.g. `"%.2f"`) — if omitted, uses default numeric formatting

Important:
- Labels are represented as label objects
- Actual glyph rendering is deferred to a later feature
- Label positions must be deterministic

This avoids coupling axes to font systems.

---

### Orientation and placement

Axes align with the world coordinate axes:
- X-axis: along +X
- Y-axis: along +Y
- Z-axis: along +Z

No rotated axes in v0.1.

Axes are placed at:
- the minimum bound plane by default
  (e.g. x-axis lies at y = ymin, z = zmin)

This default is:
- explicit
- documented
- overridable

---

### Validation rules

Axes are invalid if:
- Bounds are degenerate
- Bounds exceed `Scene.world_bounds` (must be equal to or a subset of)
- Tick values lie outside bounds
- Requested axes are empty
- Material reference is missing or invalid
- Referenced material is not a SolidMaterial (ScalarMappedMaterial not allowed for axes)

Validation must:
- identify axis bundle id
- explain which axis failed
- fail early

---

### Determinism guarantees

Frustum guarantees:
- Same bounds + tick spec → same axis geometry
- Tick placement independent of resolution
- No camera-dependent axis behavior

Axes do not reflow based on view angle.

---

### Serialization and schema

- Axis bundles serialize as part of the Scene
- Expanded geometry is not serialized
- Geometry generation occurs in the core

This keeps scenes compact and stable.

---

### Renderer contract

From the renderer's perspective, axes expand into ordinary geometry primitives (Lines) and are rendered identically to any other geometry. No special-casing of axes in the rendering pipeline.

---

### Cross-language parity

Python and R frontends must:
- expose identical axis parameters
- generate identical axis bundles
- produce identical expanded geometry

No frontend-specific defaults.

---

### Explicit non-goals

This feature does not include:
- Smart tick labeling
- Log scales
- Polar axes
- 2D plot framing
- Axis snapping to camera
- Interactive hiding or dragging

Those are conscious exclusions.

---

### Success criteria

This feature is complete when:
- Axes can be added or omitted explicitly
- Tick placement is predictable and reproducible
- Axes render as ordinary geometry
- Axes do not interfere with camera or materials
- Python and R produce identical axes

---

### Rationale

Axes are the primary source of scientific interpretation errors when handled implicitly.

By making axes:
- explicit
- geometric
- deterministic

Frustum ensures that spatial reference is intentional, not emergent.

---

### Forward references

- Feature 006: Lighting (minimal, fixed model)
- Feature 007: Rendering pipeline
- Feature 008: Text rendering (labels)
