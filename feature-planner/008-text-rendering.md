# Frustum — Feature Prompt 008

## Text Rendering (Labels)

### Purpose

Define how text labels are rendered in Frustum v0.1, constrained by the renderer contract (007) and axes specification (005).

Text in Frustum is explicit geometry, not a rendering primitive.

---

### Scope

This feature defines:
- Label representation
- Text geometry generation
- Font specification
- Rendering behavior

This feature does not define:
- Rich text
- Text editing
- Dynamic text layout
- Internationalization
- Custom fonts (v0.1)

---

### Design constraints (from 005, 007)

- Labels are semantic placeholders in axes (005)
- Labels must be deterministic (007)
- Labels must not introduce hidden state (007)
- Labels are unlit (006)
- Labels inherit axis material (005)

---

### Core design rule (frozen)

**Text is geometry.**

Labels are converted to renderable geometry before the renderer sees them. The renderer does not have a "text primitive" — it sees only the geometry that text expands to.

---

### Label representation

Labels in the scene are semantic objects:

```
Label {
    text: string
    position: vec3 (world space)
    material_id: string
}
```

Labels are generated from:
- Axes tick labels (via LabelSpec)
- Future: standalone labels (deferred)

---

### Text geometry generation

**v0.1 approach:** Texture quads

Each label becomes:
- A textured quad (2 triangles)
- Positioned at label.position
- Oriented toward camera (billboard)
- Sized in logical pixels

**Billboarding rule (frozen):** Labels always face the camera. This is the only camera-dependent behavior allowed in Frustum v0.1.

---

### Font specification (v0.1)

**Single built-in font.**

v0.1 uses a single, embedded monospace font:
- Fixed metrics
- ASCII printable characters (0x20–0x7E)
- Pre-rasterized to texture atlas

No font selection in v0.1. Font customization is deferred.

**Font rule (frozen):** The built-in font is the only font available. Scenes cannot specify fonts.

---

### Label object (expanded)

When axes expand, labels become:

```
ExpandedLabel {
    text: string
    position: vec3
    size: float (logical pixels, height)
    material_id: string
}
```

Default size: inherited from a scene-level or axes-level setting (deferred; use fixed default for v0.1).

---

### LabelSpec integration (from 005)

LabelSpec in AxisBundle:
- `show`: boolean — whether to generate labels
- `offset`: vec3 — world-space offset from tick position
- `format`: string — printf-style format for tick values

Label generation:
1. For each tick value, format using `format` (or default)
2. Position at tick position + offset
3. Assign axis material_id
4. Generate ExpandedLabel

---

### Rendering behavior

The renderer must:
- Render label quads as ordinary textured geometry
- Apply billboarding (face camera)
- Use label material color (no lighting)
- Respect depth testing (labels can be occluded)

The renderer must not:
- Apply lighting to labels
- Special-case label geometry
- Auto-scale labels based on distance
- Hide overlapping labels

**No collision avoidance.** Labels render exactly as specified.

---

### Texture atlas

The built-in font is stored as a texture atlas:
- Single texture containing all glyphs
- Known UV coordinates per character
- Embedded in renderer (not in Scene)

The atlas is:
- Deterministic (same across all platforms)
- Version-controlled with renderer
- Not user-configurable in v0.1

---

### Text sizing

Label size is specified in logical pixels (consistent with 007).

```
physical_size = logical_size × pixel_ratio
```

Size refers to text height. Width is derived from:
- Character count
- Fixed character width (monospace)

---

### Validation rules

Labels are invalid if:
- Text is empty
- Text contains non-ASCII characters (v0.1)
- Position contains NaN or Inf
- Size ≤ 0
- Material reference is invalid

Validation errors must:
- Identify the label (by text content or index)
- Explain why it is invalid
- Fail during expansion, before rendering

---

### Determinism guarantees

Given:
- Same text
- Same position
- Same size
- Same camera

Label rendering must be:
- Identical across runs
- Identical across platforms (within FP tolerance)
- Independent of other scene content

---

### Depth and occlusion

Labels participate in depth testing:
- Labels behind geometry are occluded
- Labels in front render over geometry
- No special depth bias

**No depth hacks.** Labels are geometry.

---

### Transparency

Label quads may have transparency:
- Glyph background is transparent
- Glyph foreground uses material color
- Alpha blending applies

Transparency ordering limitations from 007 apply.

---

### Cross-language parity

Python and R frontends must:
- Generate identical label text from format strings
- Produce identical label positions
- Not introduce frontend-specific formatting

---

### Explicit non-goals

- TrueType/OpenType font loading
- Unicode beyond ASCII
- Text wrapping
- Multi-line labels
- Outlined text
- Text shadows
- Distance-based scaling
- Label collision avoidance
- Annotations with leaders
- Rich text (bold, italic)

---

### Success criteria

Feature 008 is complete when:
- Axis labels render correctly
- Labels billboard toward camera
- Labels use axis material color
- Label positions match LabelSpec
- Format strings produce correct text
- Labels are occluded by geometry correctly
- No text-specific renderer code paths (labels are geometry)

---

### Rationale

Text rendering is a notorious source of complexity and non-determinism.

Frustum's approach:
- Treats text as geometry
- Uses a single embedded font
- Avoids runtime font rendering
- Defers customization to future versions

This keeps v0.1 tractable while enabling axis labels — the primary use case for scientific figures.

---

### Forward references

- Feature 009: Standalone labels (deferred)
- Feature 010: Custom fonts (deferred)
- Implementation: texture atlas in Rust, billboard shader in WGSL
