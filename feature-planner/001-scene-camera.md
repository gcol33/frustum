# Frustum — Feature Prompt 001

## Canonical Scene and Camera Model

### Purpose

Define the canonical scene representation used by Frustum.
This feature establishes the single source of truth for rendering intent across Rust, Python, R, and future tooling.

Once this feature is frozen, it must not change in incompatible ways.

---

### Scope

This feature defines:
- The Scene object
- The Camera object
- Coordinate conventions
- Validation rules
- Serialization format

This feature does not include:
- rendering
- geometry primitives (deferred to Feature 002)
- lighting
- axes definition (deferred to Feature 005; axes may exist in `objects`)
- interaction

---

### Design requirements (non-negotiable)

- Scene must be fully explicit
- No hidden global state
- No implicit defaults beyond documented ones
- Scene must be serializable
- Scene must be language-agnostic
- Scene must be immutable once validated

---

### Core objects

#### 1. Scene

A Scene represents a complete, self-contained description of a 3D figure.

Required fields:
- `camera`: Camera
- `world_bounds`: Axis-aligned bounding box
- `objects`: list[AnyRenderable]
- `materials`: list[Material] (defined in Feature 004)
- `version`: schema version identifier (namespaced string, e.g. `"frustum/scene/v1"`)

Constraints:
- A scene with no objects is valid
- A scene with no materials is valid (but objects cannot render without materials)
- All objects must lie within world_bounds
- Scene must not reference external state

Spatial semantics:
- `world_bounds` defines the authoritative spatial extent of the scene. All geometry, including Axes, must lie within these bounds.
- All geometry is defined in world coordinates prior to camera projection.
- No per-object transforms in v0.1. Geometry positions are world-space, not local-space.

Mutability:
- Objects are treated as immutable once the Scene is validated.
- Scene updates require constructing a new Scene.

Note: The structure of `AnyRenderable` is intentionally undefined in Feature 001 and specified in Feature 002. The structure of `Material` is defined in Feature 004.

---

#### 2. Camera

The camera is fully explicit and must not depend on implicit conventions.

Required fields:
- `eye`: vec3 (camera position)
- `target`: vec3 (look-at point)
- `up`: vec3 (orientation)
- `projection`: enum { `perspective`, `orthographic` }
- `near`: positive float
- `far`: positive float, far > near

Projection-specific fields:

**Perspective**
- `fov_y`: vertical field of view (degrees)

**Orthographic**
- `view_height`: world units visible vertically (horizontal extent derived from aspect ratio)

Constraints:
- eye ≠ target
- up must not be collinear with view direction
- All numeric values must be finite
- Camera definition is right-handed and documented

---

### Coordinate system

Frustum uses:
- Right-handed coordinate system
- +X → right
- +Y → up
- −Z → forward (into the scene)
- +Z → toward the viewer

Camera looks down −Z in its local space.

This matches the OpenGL convention and is consistent with standard GPU math.

This convention must be documented and enforced consistently.

---

### Defaults (minimal and frozen)

Defaults are allowed only where necessary:
- Default `up` = (0, 1, 0)
- Default `near` = 0.01
- Default `far` = 1000.0

All defaults must be:
- documented
- identical across Python and R
- encoded in the schema, not hidden in code

---

### Validation rules

Validation occurs before rendering.

A scene is invalid if:
- Any required field is missing
- Any vector contains NaN or Inf
- near >= far
- eye == target
- up is collinear with view direction
- Any object exceeds world bounds
- Schema version is unsupported

Validation errors must:
- name the offending field
- explain why it is invalid
- fail fast

---

### Serialization

- Scene must serialize to JSON
- JSON must validate against a schema derived from TypeScript definitions
- Rust must treat deserialized scenes as immutable

Binary buffers (e.g. geometry) are out of scope for this feature.

---

### Cross-language parity

Python and R frontends must:
- expose identical fields
- use identical defaults
- produce semantically equivalent JSON for equivalent scenes

A canonical JSON serialization (sorted keys, fixed formatting) MUST be used for testing and reference comparisons.

No frontend-specific behavior is allowed.

---

### Success criteria

This feature is complete when:
- A minimal scene with a camera can be created in Python and R
- Both serialize to semantically equivalent JSON (canonical for tests)
- Validation catches malformed cameras reliably
- The scene can be passed to the renderer without modification
- The schema can be frozen as `frustum/scene/v1`

---

### Explicit non-goals

This feature does not include:
- automatic camera fitting
- camera interaction
- convenience constructors beyond simple helpers
- scene mutation after validation

---

### Rationale

The scene + camera contract is the foundation of determinism.
If this feature is unstable, every later feature becomes fragile.

This feature must favor clarity and rigidity over convenience.

---

### Next

Feature Prompt 002: Geometry Primitives
