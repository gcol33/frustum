# Frustum — Document 010

## Implementation Notes (Pre-Implementation Review)

### Purpose

This document captures findings from a pre-implementation review of Frustum's frozen design against modern GPU rendering practices.

These are **implementation guidance**, not spec changes.

---

### Review summary

| Category | Blockers | Warnings | Notes |
|----------|----------|----------|-------|
| wgpu/WebGPU alignment | 0 | 3 | 3 |
| Rust ecosystem | 0 | 1 | 2 |
| Scene serialization | 0 | 2 | 0 |
| Scientific viz prior art | 0 | 0 | 3 |
| Cross-platform determinism | 0 | 4 | 0 |
| **Total** | **0** | **10** | **8** |

**Verdict:** No blockers. Design is sound. Proceed with implementation.

---

## Findings by category

### 1. wgpu/WebGPU alignment

#### Renderer-as-pure-function maps cleanly to wgpu
- **Severity:** note
- **Action:** none needed

`Render(Scene, RenderConfig) → Image` maps onto wgpu's "encode → submit → readback" model.

#### Readback is a first-class step
- **Severity:** note
- **Action:** follow wgpu's capture example pattern

Use `copy_texture_to_buffer` → `MAP_READ` for PNG output. Treat readback as explicit, not an afterthought.

#### Backend differences exist
- **Severity:** warning
- **Action:** expose backend override for debugging; record backend/adapter in render metadata

Don't rely on implicit backend selection. Log which backend was used for reproducibility diagnostics.

#### NDC coordinate convention trap
- **Severity:** warning
- **Action:** codify one camera/projection pipeline; test early; don't reuse OpenGL matrices without conversion

wgpu/WebGPU NDC differs from OpenGL. The frozen OpenGL convention (−Z forward) is fine semantically, but the projection matrix must account for wgpu's NDC (Y-up, Z in [0,1] not [-1,1]).

**Implementation note:** Apply the standard OpenGL-to-wgpu correction matrix. Lock golden tests immediately after camera works.

#### Avoid shader f16 and precision knobs
- **Severity:** warning
- **Action:** default to f32 everywhere in WGSL; do not enable `SHADER_F16` in v0.1

Optional precision features increase cross-device variability.

#### Buffer layouts are standard
- **Severity:** note
- **Action:** standardize a small set of vertex layouts (Points, Lines, Mesh)

Your primitives map to a few vertex formats. wgpu likes explicit, stable layouts. This is idiomatic.

---

### 2. Rust ecosystem conventions

#### Math crate choice
- **Severity:** note
- **Action:** pick one (glam or nalgebra); isolate behind own math module

`glam` is common in rendering; `nalgebra` is common in scientific computing. Either works. **Don't leak the choice into public API.**

#### PNG output
- **Severity:** note
- **Action:** use `image` crate; don't optimize prematurely

Standard pattern: GPU readback to RGBA8 → `image` crate encode. Performance is fine for v0.1.

#### Determinism and hidden fast-math
- **Severity:** warning
- **Action:** avoid crates that hide SIMD/fast-math behind feature flags; keep CPU-side math explicit

Lock down any crate features that affect floating-point behavior. Keep marching cubes and camera transforms deterministic on CPU side.

---

### 3. Scene serialization

#### Binary companion for large geometry
- **Severity:** warning
- **Action:** keep canonical JSON for semantics; plan `.json` + `.bin` container for heavy arrays

glTF's pattern (JSON manifest + binary buffers) is proven. For v0.1, pure JSON is acceptable. For large meshes/volumes, design the split now.

**Suggested structure:**
```
scene.frustum.json    # semantic structure
scene.frustum.bin     # positions, indices, scalars (optional)
```

#### Schema generation path
- **Severity:** warning
- **Action:** make TypeScript the schema source-of-truth; generate Rust types from it or validate against it

Generating schema from Rust leaks Rust-specific constraints. TypeScript → JSON Schema is cleaner and language-agnostic.

---

### 4. Scientific viz prior art

#### VTK pipeline model validates design
- **Severity:** note
- **Action:** none needed

VTK's "pipeline terminating at rendering boundary" mirrors Frustum's generators → primitives → renderer structure. Your contract is stricter and more explicit.

#### Right to reject VTK/PyVista complexity
- **Severity:** note
- **Action:** none needed

PyVista's "grunt work wiring" and VTK's dynamic behaviors are deliberate rejections. Frustum's "no hidden state, no auto magic" is consistent with reproducibility goals.

#### Document pipeline stages for VTK users
- **Severity:** note
- **Action:** add documentation mapping Frustum concepts to VTK mental models

Help users coming from VTK/ParaView understand: generator → primitive → material → render.

---

### 5. Cross-platform determinism

#### WGSL undefined behavior is real
- **Severity:** warning
- **Action:** keep WGSL simple; avoid edge-case math; add shader conformance tests

Research shows shader UB causes divergence. Avoid relying on unspecified behavior. Test shaders across backends.

#### Floating-point divergence
- **Severity:** warning
- **Action:** avoid f16; avoid fast-math; use tolerant golden test metrics

Your "topology stable, perceptually stable, not bitwise identical" contract is correct for cross-backend GPU rendering.

#### Text rendering will be a hotspot
- **Severity:** warning
- **Action:** pin font rasterization strategy tightly; consider CPU raster for labels

Font rendering is notoriously variable across platforms. Current placeholder approach (008) is correct. When implementing real text, control rasterization carefully.

#### Metal vs Vulkan vs DX12 gotchas
- **Severity:** warning
- **Action:** test on all three; expect minor differences; don't assume uniformity

(Findings truncated but the pattern is clear: test broadly, expect variance within tolerance.)

---

## Actionable summary

### Before writing code

| Action | Priority |
|--------|----------|
| Decide math crate (glam vs nalgebra) | High |
| Design projection matrix with wgpu NDC correction | High |
| Define vertex buffer layouts for 3 primitive types | High |
| Set up TypeScript schema → JSON Schema pipeline | Medium |

### During implementation

| Action | Priority |
|--------|----------|
| Log backend/adapter in render metadata | High |
| Lock golden tests immediately after camera works | High |
| Keep WGSL simple; avoid precision features | High |
| Test on Vulkan, Metal, DX12 early | Medium |

### Deferred

| Action | Notes |
|--------|-------|
| Binary companion format (.json + .bin) | Design now, implement when needed |
| Font rasterization strategy | Address in text rendering implementation |

---

## Conclusion

The frozen design (001–008) is validated against modern GPU rendering practices.

No architectural changes required. Implementation can proceed with the noted guidance.
