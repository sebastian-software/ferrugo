# SIMD row-kernel boundary report

Issue: #50

## Summary

This PR records the SIMD strategy decision before introducing unsafe code.
The selected approach is an isolated future `ferrugo-simd` kernel crate with a
safe API, while `ferrugo-render` and `ferrugo-native` keep
`#![forbid(unsafe_code)]`.

The current renderer already has the intended integration point:

- `CoverageDrawBlitter` selects `OpaqueNormal`, `SourceOverNormal`,
  `NormalAlpha`, `Multiply`, and `Screen` once per draw item.
- Row counters already expose the hot families:
  `coverage_direct_row_pixels`, `coverage_source_over_row_pixels`, and
  `coverage_blend_mode_row_pixels`.
- Row functions such as `blend_source_over_normal_row_span`,
  `blend_multiply_row_span`, and `blend_screen_row_span` are the kernel
  boundary; per-pixel paths stay scalar fallbacks and edge handlers.

## Decision

See `docs/decisions/0010-simd-row-kernel-boundary.md`.

The first implementation PR after this should add a `ferrugo-simd` crate and
one source-over row kernel behind a safe function table. It should not start
with multiply/screen or a broad dispatch abstraction until the normal
source-over path proves byte parity and a measured win.

## Acceptance impact

This does not close #50. It resolves the up-front strategy constraint from the
issue:

- unsafe stays out of `ferrugo-render` and `ferrugo-native`;
- scalar render code remains the oracle;
- runtime dispatch is planned below the existing row-blitter selection;
- benchmarks can tie claims to the existing row-family counters.

## Validation

```sh
cargo fmt --check
git diff --check
```
