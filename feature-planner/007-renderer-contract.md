# Frustum — Feature Prompt 007

## Renderer Consumption & Rendering Contract

### Purpose

Define how a renderer consumes the frozen Frustum semantic model (001–005) and what it is allowed and not allowed to do.

This feature does not define how rendering is implemented.
It defines what the renderer must respect.

---

### Scope

This feature defines:
- Renderer input contract
- Allowed rendering behavior
- Explicit prohibitions
- Output guarantees

This feature does not include:
- GPU API details
- Lighting models (Feature 006)
- Performance optimizations
- Interactivity
- Shader code

---

### Core rendering contract (non-negotiable)

A Frustum renderer:
1. Consumes a validated, frozen Scene
2. Must not modify Scene semantics
3. Must not infer missing data
4. Must not introduce hidden state
5. Must treat all geometry uniformly

The renderer is a pure function:

```
Image = Render(Scene, RenderConfig)
```

---

### Renderer inputs

**Required input:**
- `Scene` (validated, immutable)
- `RenderConfig`

#### RenderConfig

RenderConfig is renderer-only configuration that does not affect Scene semantics.

Required fields:
- `width`: int — image width in logical pixels
- `height`: int — image height in logical pixels

Optional fields:
- `background_color`: RGBA — background color (default: opaque white)
- `pixel_ratio`: float — logical-to-physical pixel ratio (default: 1.0)

Constraints:
- width > 0
- height > 0
- pixel_ratio > 0
- background_color values in [0, 1]

**RenderConfig rule (frozen):** RenderConfig must not affect Scene semantics, must not change geometry, and must not change topology.

---

### Pixel and size semantics

**Logical pixel rule (frozen):** All size-related geometry attributes (`size` for Points, `width` for Lines/Curves) are specified in logical pixels. `RenderConfig.pixel_ratio` controls mapping to physical pixels.

Physical resolution:
- physical_width = width × pixel_ratio
- physical_height = height × pixel_ratio

This enables HiDPI support without changing geometry definitions.

---

### Color space

**Color space rule (frozen):** The renderer outputs sRGB color. No implicit or configurable gamma correction is applied in v0.1.

- Internal computations may use linear color space
- Output image is sRGB
- No color space configuration in RenderConfig

---

### Background transparency

**Background rule (frozen):** `RenderConfig.background_color` is RGBA; transparency is preserved in output.

- Alpha = 0 is allowed (fully transparent background)
- No automatic premultiplication
- Transparency useful for compositing in publications

---

### Geometry consumption rules

The renderer must:
- Render all geometry in world space
- Apply camera projection exactly as defined
- Respect material bindings
- Respect primitive types
- Preserve triangle winding

The renderer must not:
- Reorder geometry semantically
- Merge or split primitives
- Drop degenerate geometry silently
- Apply implicit transforms

---

### Degenerate geometry

**Degenerate geometry rule (frozen):** Degenerate geometry is valid input. The renderer must not silently remove it.

Degenerate geometry includes:
- Zero-area triangles
- Coincident vertices
- Zero-length lines
- Collapsed curves

Behavior:
- Validation allows degenerate geometry
- Renderer must attempt to render
- Rendering may result in no visible pixels
- This is correct behavior, not an error

---

### Material consumption rules

The renderer must:
- Apply SolidMaterial uniformly
- Apply ScalarMappedMaterial deterministically
- Use the documented colormap definitions
- Respect material restrictions (e.g., no scalar-mapped axes)

The renderer must not:
- Invent fallback materials
- Change color ranges
- Apply undocumented gamma correction

---

### Transparency handling

**Transparency rule (frozen):** Transparency handling is implementation-defined in v0.1. No guarantees are made for correct ordering of overlapping transparent geometry.

- Alpha < 1.0 is allowed in materials
- Depth testing applies as implemented
- No sorting or depth peeling guaranteed
- Approximate behavior is acceptable

This allows useful transparency (labels, markers, compositing) without over-constraining the renderer.

---

### Axes handling

The renderer must:
- Treat axes-expanded geometry identically to other geometry
- Not special-case axes
- Not override axis materials
- Not alter tick placement

**Axes are geometry. Full stop.**

---

### Lighting

Lighting configuration is defined in Feature 006.

The renderer must:
- Consume lighting if present in the Scene
- Apply lighting deterministically
- Not inject lighting heuristics

007 does not define where lighting lives (Scene vs RenderConfig). Feature 006 is authoritative.

---

### Determinism guarantees

Given:
- the same Scene
- the same RenderConfig
- the same renderer version

The renderer must produce:
- identical topology
- identical camera framing
- perceptually stable output

**The renderer is not required to produce bitwise-identical pixels across hardware.**

Cross-platform floating-point differences are acceptable. Visual stability is required.

---

### Error handling

The renderer must:
- Fail if Scene validation failed
- Fail if required inputs are missing
- Fail loudly on unsupported Scene versions

The renderer must not:
- Silently ignore errors
- Partially render invalid scenes
- "Best-effort" guess user intent

**Fail-fast rule (frozen):** Rendering errors must be explicit and reproducible.

---

### Versioning and compatibility

- Renderer must declare supported Scene schema versions
- Rendering an unsupported version is an error
- Scene version checks happen before rendering

```
if scene.version not in renderer.supported_versions:
    raise UnsupportedVersionError(scene.version)
```

---

### Output guarantees

At minimum, a renderer must support:
- PNG output
- Fixed resolution (as specified in RenderConfig)
- Fixed background color (as specified in RenderConfig)
- sRGB color space

Optional (future-scoped):
- PDF output
- SVG output

---

### Explicit non-goals

The renderer must not:
- Modify Scene objects
- Auto-fit camera
- Auto-add axes
- Auto-scale geometry
- Optimize away "invisible" geometry
- Inject lighting heuristics
- Apply smart defaults

**No magic. No inference. No implicit behavior.**

---

### Validation

RenderConfig is invalid if:
- width or height ≤ 0
- pixel_ratio ≤ 0
- background_color values outside [0, 1]

Validation errors must:
- Identify the offending field
- Explain why it is invalid
- Fail before rendering begins

---

### Cross-language parity

Python and R frontends must:
- Expose identical RenderConfig constructors
- Use identical defaults
- Produce semantically equivalent render calls

---

### Success criteria

Feature 007 is complete when:
- A renderer can consume any valid Scene v0.1
- Rendering behavior is fully predictable from Scene + RenderConfig
- No semantics live in renderer code
- Rendering failures are explicit and reproducible
- Determinism guarantees are testable

---

### Rationale

Most plotting systems leak semantics into the renderer.

Frustum forbids this.

By freezing what rendering is allowed to do, Frustum:
- Preserves reproducibility
- Enables multiple renderer implementations
- Avoids legacy behavior creep
- Separates meaning from pixels

The renderer is a pure function from (Scene, RenderConfig) → Image.

---

### Coordinate spaces (documented)

Frustum uses explicit coordinate space ownership:

| Space | Owner | Description |
|-------|-------|-------------|
| World space | Scene (001, 002) | All geometry positions |
| View space | Camera transform | World → camera-relative |
| Clip space | Projection | View → homogeneous clip |
| NDC | Perspective divide | Clip → normalized device |
| Framebuffer | Viewport transform | NDC → pixels |

**Object space is collapsed.** There are no per-object transforms in v0.1. All geometry is specified directly in world space.

This prevents coordinate-space ambiguity and simplifies the renderer contract.

---

### Render pass sequence (fixed)

The renderer executes a fixed sequence of logical passes:

1. **Geometry pass** — rasterize all primitives
2. **Lighting pass** — apply Lambertian shading (if Light present)
3. **Color mapping pass** — apply scalar → color mapping
4. **Resolve pass** — composite to final framebuffer

This sequence is:
- Fixed (not configurable)
- Deterministic
- Documented

Implementations may fuse or reorder passes internally, but the semantic effect must match this sequence.

**No clever shortcuts.** The pass sequence prevents renderer-side "optimizations" that alter semantics.

---

### Renderer capability declaration (reserved)

**Reserved for future use.**

A renderer may declare capabilities:

```
RendererCapabilities {
    supports_transparency: boolean
    supports_lighting: boolean
    supports_scalar_mapping: boolean
    max_vertices: int
    max_texture_size: int
}
```

This enables:
- Honest limits (no silent degradation)
- Multiple renderer implementations
- Meaningful test coverage

v0.1 does not require capability declaration. The concept is reserved to prevent future design conflicts.

---

### Intent vs realization

Frustum separates:

| Concept | Description | Location |
|---------|-------------|----------|
| **Intent** | What should be rendered | Scene |
| **Realization** | How the GPU renders it | Renderer |

The Scene is intent. The Renderer is realization.

This separation:
- Justifies Frustum's strictness
- Enables multiple backends
- Prevents semantic leakage

**The Scene says what. The Renderer says how. Neither may invade the other's domain.**

---

### Explicit rejections (frozen)

The following patterns are explicitly rejected. They must not be introduced in any Frustum version.

#### Scene graph mutation

Rejected:
- Node hierarchies
- Transform propagation
- Incremental updates
- Stateful mutation

Frustum is **declarative, immutable, reconstructive**. Scene changes require constructing a new Scene.

#### Entity Component Systems (ECS)

Rejected:
- Dynamic entity creation
- Component mutation
- Runtime-centric architecture

ECS is wrong for scientific rendering. Frustum's schema-based model is superior for reproducibility.

#### Automatic batching / instancing

Rejected:
- Merging primitives for performance
- Reordering user intent
- Hiding topology changes

Internal batching is allowed. Semantic batching is forbidden. The user's primitives must remain distinct.

---

### Forward references

- Feature 006: Lighting (input to renderer, not renderer logic)
- Feature 008: Text rendering (labels)
- Implementation: Rust + wgpu

---

### Frozen decisions (007)

| Decision | Resolution |
|----------|------------|
| Color space | sRGB output, fixed |
| Background | RGBA allowed |
| DPI / sizes | Logical pixels + pixel_ratio |
| Degenerate geometry | Allowed, not dropped |
| Lighting placement | Deferred to Feature 006 |
| Transparency ordering | Renderer-defined, limited guarantees |
