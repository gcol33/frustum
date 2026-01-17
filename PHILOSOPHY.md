# Frustum Philosophy

## The Problem (Quietly Unsolved)

Scientific 3D figures are fragile, implicit, and hard to reproduce — not because GPUs are hard, but because semantics are never frozen.

Everyone feels this. Few articulate it.

Most tools jump straight to pixels. Frustum stops at intent, then builds outward.

---

## What Frustum Actually Is

Frustum is not "another plotting library."

It is a system where:
- **Meaning** is frozen
- **Scope** is frozen
- **Responsibility boundaries** are frozen
- **Failure modes** are frozen

The design itself is an asset — a reference architecture for modern GPU-first scientific tooling.

---

## The Inversion

Most projects chase flash first, then struggle with correctness.

Frustum inverts this:
- Boring parts first
- Constraints first
- Guarantees first

When flash is finally added (node-based materials, nicer lighting, WebGPU demos), it lands differently. It feels earned.

**Frustum is not valuable because it's flashy. It becomes flashy because it's correct.**

---

## What Makes This Hard

Anyone can write a renderer.

Few can:
- Design this cleanly
- Know what to exclude
- Hold the line

The explicit non-goals are as important as the features:
- Not a plotting DSL
- Not a dashboard tool
- Not a visualization playground
- No legacy backends
- No implicit state
- No magic

Restraint is the feature.

---

## The Guarantees

Frustum guarantees:
- Same scene + same topology + same camera = same figure meaning
- No hidden state
- No order-dependent rendering (unless documented)
- Errors are early, explicit, and informative

Frustum does not guarantee:
- Bitwise identical pixels across GPUs

This distinction matters. Topology stability is the promise, not pixel identity.

---

## Why Separation Matters

Frustum separates:
- **Geometry generation** (marching cubes, parametric surfaces) from **geometry consumption** (rendering)
- **Scene semantics** (what you mean) from **presentation** (how it looks)
- **Core** (frozen, stable) from **extensions** (open, evolvable)

Most visualization stacks blur these lines. Blurring them is why scientific figures break.

---

## The Mathematical Stance

Frustum makes explicit mathematical commitments:
- Marching cubes disambiguation: asymptotic decider
- Vertex interpolation: exact linear formula, world-space
- Normals: gradient-based, not face-based
- Preconditions: continuous scalar fields, band-limited sampling

These are not implementation details. They are part of the algorithm's definition.

---

## Cross-Language Parity

Python and R frontends are feature-equivalent. Both compile to the same scene schema.

- No language-specific features
- No divergence in defaults
- If a feature exists in one, it exists in the other — or it's removed

This is expensive to maintain. It's also the only way to be trustworthy.

---

## Flash at the Edges

Flash is allowed only where it is reversible.

If you can turn it off and the figure is still scientifically valid, it's allowed.
If turning it off breaks meaning, it doesn't belong.

Safe flash layers:
1. **Geometry generators** — Dual Contouring, SDF, parametric (output is still Mesh)
2. **Materials** — shading, contours, curvature coloring (topology unchanged)
3. **Lighting presets** — explicit, documented, opt-in
4. **Camera choreography** — scripted paths, turntables, multi-view grids
5. **Presentation surfaces** — PNG, SVG, WebGPU viewer, video (same schema)

Forbidden:
- Auto camera fitting
- Hidden lighting defaults
- Adaptive LOD without user intent
- Magic smoothing
- Implicit color tricks

---

## The Uncomfortable Truth

Frustum will not be widely adopted fast.

It will be adopted by people who:
- Have been burned by implicit state
- Need reproducible figures for publication
- Value correctness over convenience
- Understand that "boring" is a feature

That's a small audience. It's also the right audience.

---

## Success Criteria

Frustum is successful if:
- A scientist can generate a 3D figure reproducibly in Python or R
- The figure is acceptable in a paper without apology
- The API feels boring, explicit, and predictable
- The system can be maintained without architectural rewrites

---

## The Real Value

Frustum is not valuable because it ships pixels.

It is valuable because it ships **judgment frozen into a system**.

- Judgment about what to include
- Judgment about what to exclude
- Judgment about where boundaries belong
- Judgment about what guarantees matter

That's rare. That's the asset.
