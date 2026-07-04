# 0010. SIMD row-kernel boundary

Date: 2026-07-03

## Status

Accepted

## Context

The renderer row-blitter work already selects a small set of compositing paths
once per draw item. That makes the next optimization boundary a row-kernel
swap below the existing blitter selection rather than another routing layer.

The render crates currently use `#![forbid(unsafe_code)]`. Direct NEON,
AVX2, AVX-512, or WASM SIMD intrinsics require `unsafe`, and allowing unsafe
inside `ferrugo-render` would widen the review surface across parser,
display-list, rasterizer, and text code that should remain safe Rust.

## Decision

Use an isolated SIMD kernel crate for handwritten architecture-specific row
kernels.

The boundary is:

- `ferrugo-render` keeps scalar row kernels as the reference implementation and
  byte-parity oracle.
- `ferrugo-render` keeps `#![forbid(unsafe_code)]`.
- `ferrugo-native` keeps `#![forbid(unsafe_code)]`.
- `ferrugo-simd` owns the safe row-kernel API and may later contain reviewed
  unsafe intrinsics behind that API.
- Runtime dispatch happens once per process or per kernel family, and
  `ferrugo-render` only calls a safe function pointer/table selected below the
  existing row-blitter kind.
- Scalar fallback remains available on every target and is the release/debug
  test oracle.

The first kernel family should be full-row normal source-over for opaque source
color with coverage, because that is already counted by
`coverage_source_over_row_pixels` and maps directly to the current row blitter
surface. Multiply and screen row writers should follow only after the normal
source-over path has byte parity and measured wins.

## Rejected Options

### Put `std::arch` directly in `ferrugo-render`

Rejected because it removes the current crate-wide unsafe prohibition from the
largest rendering crate and makes ordinary raster changes harder to review.

### Use a safe SIMD abstraction first

Rejected for the first production SIMD path. A safe abstraction can still be
evaluated later, but this project needs exact byte parity with the current
scalar math and predictable runtime dispatch across Apple Silicon and server
ARM64. An isolated kernel crate keeps that decision reversible without
changing render crate safety policy.

### Add SIMD before a kernel boundary

Rejected because the current row blitters are already the right selection
point. SIMD should not add another independent routing model.

## Consequences

- Future SIMD implementation PRs should keep all unsafe inside
  `crates/ferrugo-simd`.
- The crate API should expose narrow row functions over RGBA byte slices, not
  PDF or renderer concepts.
- Each SIMD kernel needs scalar parity tests for edge lengths, unaligned
  prefixes/suffixes, alpha/coverage extremes, and representative random rows.
- Benchmarks should report the existing route counters alongside timing so a
  claimed SIMD win maps to the row family that actually ran.
- `ferrugo-render` and `ferrugo-native` safety posture stays unchanged.
