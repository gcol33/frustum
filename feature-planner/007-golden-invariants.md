# Frustum — Golden Render Invariants

## Perceptual Stability Contract

This document defines what "perceptually stable" means in Frustum.

These are testable invariants, not vibes.

---

## What Frustum guarantees

Given:
- Same Scene
- Same RenderConfig
- Same renderer version

The output must satisfy **all invariants below**.

---

## Invariant 1 — Topology invariance

**Guarantee:**
- Same number of rendered primitives
- Same triangle connectivity
- Same vertex ordering effects

**Test:** Compare rendered geometry count and structure.

**Violation:** Missing or extra geometry between renders.

---

## Invariant 2 — Camera invariance

**Guarantee:**
- Projection parameters identical
- Object placement identical
- No drift, no auto-adjustment

**Test:** Overlay two renders; object positions must align exactly.

**Violation:** Objects shift position between renders.

---

## Invariant 3 — Color mapping invariance

**Guarantee:**
- Scalar → color mapping identical
- Colormap lookup stable
- Clamp behavior identical

**Test:** Sample specific scalar values; verify color output.

**Tolerance:** Differences allowed only within FP tolerance (±1/255).

**Violation:** Same scalar produces different colors.

---

## Invariant 4 — Spatial coherence

**Guarantee:**
- Object edges align to the same pixel neighborhoods
- No jitter across runs
- No resolution-dependent shifts (beyond pixel_ratio scaling)

**Test:** Edge detection on multiple renders; compare edge positions.

**Violation:** Edge positions vary between identical renders.

---

## Invariant 5 — Lighting stability

**Guarantee:**
- Same normals + same light → same shading
- No render-order-dependent shading
- No resolution-dependent shading variation

**Test:** Render mesh with known normals; verify shading values.

**Violation:** Shading varies between identical renders.

---

## Allowed variation

These differences are **acceptable** and do not violate stability:

| Allowed | Reason |
|---------|--------|
| Minor sub-pixel differences | Rasterization implementation details |
| Minor floating-point noise | Cross-platform FP behavior |
| Vendor-specific AA edge cases | GPU driver differences |
| ±1 LSB color differences | FP → integer conversion |

---

## Not allowed

These differences **violate** the stability contract:

| Not Allowed | Why |
|-------------|-----|
| Color banding changes | Indicates colormap instability |
| Missing faces | Topology violation |
| Flipped normals | Geometry corruption |
| Depth ordering inversions (opaque) | Z-buffer instability |
| Object position drift | Camera or transform instability |
| Inconsistent clipping | Near/far plane instability |

---

## How to test perceptual stability

### Required tests

1. **Pixel-wise diff with tolerance**
   - Compare two renders pixel by pixel
   - Allow ±threshold per channel
   - Fail if any pixel exceeds threshold

2. **Structural similarity (SSIM)**
   - Compute SSIM between renders
   - Require SSIM ≥ 0.99
   - Lower values indicate structural differences

3. **Edge detection consistency**
   - Apply Canny or similar edge detector
   - Compare edge maps
   - Edges must align within 1 pixel

4. **Histogram comparison**
   - Compare color channel histograms
   - Large histogram differences indicate color instability

### Pass criteria

A render pair passes stability testing if:
- All pixel differences below threshold
- SSIM ≥ 0.99
- No structural differences detected
- Histograms statistically similar

---

## Thresholds (v0.1)

| Metric | Threshold | Rationale |
|--------|-----------|-----------|
| Max pixel diff | 5/255 per channel | Allows FP noise |
| SSIM | ≥ 0.99 | Strict structural match |
| Edge alignment | ±1 pixel | Sub-pixel rasterization |
| Histogram χ² | p > 0.95 | Color distribution stable |

These thresholds are conservative. Tighten as implementation matures.

---

## Explicit non-guarantees

Frustum does **not** guarantee:

| Not Guaranteed | Reason |
|----------------|--------|
| Bitwise identical images | FP and rasterization vary |
| Identical transparency blending | Order-dependent, deferred |
| Identical anti-aliasing | Implementation-specific |
| Cross-version stability | Schema may evolve |
| Cross-renderer stability | Different implementations allowed |

---

## Reference image protocol

### Generation

1. Generate reference on a controlled environment
2. Document: OS, GPU, driver version, renderer version
3. Version control the reference image
4. Include Scene JSON alongside reference

### Comparison

1. Render on test environment
2. Compare against reference
3. Apply tolerance thresholds
4. Report pass/fail with metrics

### Update policy

Reference images may be updated when:
- Bug fix changes expected output
- Feature addition changes expected output
- Threshold adjustment needed

Reference updates require:
- Explicit justification
- Review of all affected tests
- Version bump documentation

---

## Test matrix

| Dimension | Values |
|-----------|--------|
| Platform | Windows, macOS, Linux |
| GPU vendor | NVIDIA, AMD, Intel, Apple Silicon |
| Resolution | 800×600, 1920×1080, 4K |
| pixel_ratio | 1.0, 2.0 |
| Background | Opaque, Transparent |

Full stability requires passing across this matrix.

---

## Failure investigation

When stability tests fail:

1. **Capture both images** — reference and test output
2. **Generate diff image** — highlight differences
3. **Compute metrics** — SSIM, max diff, histogram
4. **Check invariants** — which specific invariant failed?
5. **Document environment** — OS, GPU, driver, renderer version
6. **Bisect if needed** — find the change that broke stability

---

## Rationale

"Perceptually stable" is often hand-waved.

Frustum makes it concrete:
- Defined invariants
- Measurable thresholds
- Reproducible tests

This enables:
- Confident renderer changes
- Cross-platform validation
- Regression detection

Without this, "reproducible figures" is marketing, not engineering.
