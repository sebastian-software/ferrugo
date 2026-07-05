# Blend Rounding Unification

Date: 2026-07-05.
Issue: #150.

## Summary

The 0.4.0 SIMD span kernels introduced exact integer source-over math
(`floor((s * ca + d * (255 - ca)) / 255)`) for opaque coverage spans of 16 or
more pixels, while shorter spans stayed on the legacy floating-point path.
The two disagree on roughly 12,400 of the 16.7 million possible
(source, dest, coverage) channel combinations — including black text on a
white destination — because the float path floors a value that sits one ULP
below the exact rational at integer boundaries. Output therefore depended on
how long a batched run happened to be, and the claimed byte parity between
the kernels did not actually hold.

## Decision

The exact integer result is the canonical opaque source-over math. Spans of
every length now use the integer kernel; the floating-point scalar path
remains only for non-opaque sources, non-opaque destinations, and fractional
constant alpha. This is an intentional, one-time output change of at most one
channel step on partial-coverage pixels.

## Changes

- `ferrugo-simd`: short opaque coverage spans dispatch to the integer scalar
  kernel instead of the float path, so results are independent of run length.
- Exhaustive tests: `div_255_u16` is verified against exact division for
  every reachable value, the integer kernel against the exact rational floor
  for all (source, dest, coverage) combinations, and the runtime-dispatched
  kernels (NEON/SSE2/AVX2 where available) against the same reference across
  all coverage values. The previous hand-picked "parity vector" test that
  passed by luck is superseded.
- Golden baselines: one thumbnail-tier baseline changed
  (`financial-cashflow-statement.pdf`) and was refreshed with this rationale
  in its manifest note.
- New high-resolution golden tier: `fixtures/native-golden-highdpi-manifest.tsv`
  renders `technical-large-coordinate-plan.pdf` at `max_edge = 1024`, where
  long partial-coverage spans actually route through the SIMD kernels. The
  thumbnail-scale goldens alone never exercised that path.

## Follow-ups

- The per-pixel float blend (`source_over_opaque_channel`) still serves
  routes whose coverage is a non-quantized fraction (sampled fills, constant
  alpha). Those routes are selected per draw item, not per run, so they do
  not reintroduce run-length dependence.
