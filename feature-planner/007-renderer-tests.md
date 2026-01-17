# Frustum — Renderer Test Checklist

## Derived from Feature 007

This checklist defines non-negotiable renderer tests.

**A renderer is non-compliant if any of these fail.**

---

## Scene consumption

- [ ] Renderer rejects unsupported Scene versions
- [ ] Renderer rejects unvalidated Scenes
- [ ] Renderer does not mutate Scene objects
- [ ] Renderer fails loudly on missing required inputs
- [ ] Renderer version check occurs before any rendering

---

## Geometry handling

- [ ] Points render at correct logical pixel size
- [ ] Lines render at correct logical pixel width
- [ ] Curves expand to Lines correctly
- [ ] Mesh triangle winding preserved
- [ ] Degenerate triangles do not cause errors
- [ ] Zero-length lines do not crash renderer
- [ ] Coincident vertices handled gracefully
- [ ] Empty geometry lists render without error
- [ ] Empty Scene (no objects) renders background only

---

## Camera & projection

- [ ] Perspective projection matches spec (fov_y in degrees)
- [ ] Orthographic projection respects view_height
- [ ] Near clipping plane respected
- [ ] Far clipping plane respected
- [ ] Eye ≠ target enforced
- [ ] Up vector not collinear with view direction
- [ ] No auto camera fitting occurs
- [ ] Camera does not drift between identical renders

---

## Materials

- [ ] SolidMaterial renders uniform color
- [ ] ScalarMappedMaterial respects defined range
- [ ] Clamp = true clamps out-of-range scalars
- [ ] Clamp = false: out-of-range renders missing_color
- [ ] Missing scalar values use missing_color
- [ ] ScalarMappedMaterial rejected for Axes (validation error)
- [ ] No fallback materials invented
- [ ] Alpha channel respected in materials

---

## Colormaps

- [ ] viridis colormap matches reference
- [ ] plasma colormap matches reference
- [ ] inferno colormap matches reference
- [ ] magma colormap matches reference
- [ ] cividis colormap matches reference
- [ ] Colormap lookup is deterministic
- [ ] Scalar → color mapping identical across runs

---

## Axes

- [ ] Axes expand to Lines only
- [ ] Axes rendered with assigned SolidMaterial
- [ ] Axes not special-cased in rendering pipeline
- [ ] Tick placement deterministic
- [ ] Tick values respect axis bounds
- [ ] Auto tick count produces consistent results
- [ ] Axis bounds ⊆ Scene.world_bounds enforced
- [ ] Label placeholders generated correctly

---

## Transparency & background

- [ ] RGBA background preserved in PNG output
- [ ] Transparent background (alpha = 0) works
- [ ] Alpha < 1 geometry renders without error
- [ ] Overlapping transparent geometry does not crash
- [ ] No ordering guarantees assumed (test does not depend on order)
- [ ] Opaque geometry depth-tested correctly

---

## Lighting

- [ ] No lighting specified → flat colors (no shading)
- [ ] Light present → Lambertian shading applied to Meshes
- [ ] Points render unlit regardless of light
- [ ] Lines render unlit regardless of light
- [ ] Normals respected in shading calculation
- [ ] Light direction normalized
- [ ] Light intensity scales shading correctly
- [ ] Light enabled = false → no shading
- [ ] Lighting does not alter geometry topology

---

## Output

- [ ] PNG output produced successfully
- [ ] Output resolution matches RenderConfig.width × height
- [ ] pixel_ratio applied correctly (physical pixels)
- [ ] Background color applied exactly as specified
- [ ] sRGB output (no unexpected gamma)
- [ ] Output file is valid PNG

---

## Determinism

- [ ] Same Scene + RenderConfig → perceptually identical output
- [ ] No jitter across consecutive renders
- [ ] No resolution-dependent position shifts
- [ ] Colormap values stable across runs
- [ ] Camera framing identical across runs

---

## Error handling

- [ ] Invalid Scene version → explicit error
- [ ] Missing required field → explicit error
- [ ] Invalid RenderConfig → explicit error before rendering
- [ ] NaN in geometry → validation error
- [ ] Inf in geometry → validation error
- [ ] Out-of-bounds indices → validation error
- [ ] No silent failures
- [ ] No partial renders of invalid scenes

---

## RenderConfig validation

- [ ] width ≤ 0 → error
- [ ] height ≤ 0 → error
- [ ] pixel_ratio ≤ 0 → error
- [ ] background_color values outside [0,1] → error

---

## Cross-platform

- [ ] Render completes on Windows
- [ ] Render completes on macOS
- [ ] Render completes on Linux
- [ ] Output perceptually stable across platforms
- [ ] Minor FP differences acceptable, structural differences not

---

## Test methodology

Each test should:
1. Construct a minimal Scene exercising the feature
2. Render with known RenderConfig
3. Compare output against reference or validate properties
4. Report pass/fail with diagnostic information

**Reference images** should be generated once and version-controlled.

**Tolerance thresholds** for perceptual comparison:
- SSIM ≥ 0.99 for structural similarity
- Max pixel difference ≤ 5/255 for color accuracy
- No missing geometry (topology check)
