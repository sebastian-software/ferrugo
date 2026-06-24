# 0087: Font Hinting Glyph Cache And Subpixel Policy

Status: done
Phase: 15
Size: medium
Depends on: 0086

## Goal

Improve text fidelity and repeat-render performance with an explicit glyph
hinting, caching, and subpixel positioning policy.

## Scope

- Measure current glyph rasterization drift against PDFium.
- Add bounded glyph cache keys for font, size, transform, glyph id, and color
  policy where needed.
- Define whether subpixel positioning is rounded, preserved, or approximated.
- Document memory limits and cache eviction behavior.

## Non-Goals

- Depend on native platform font APIs.
- Add unbounded glyph atlases.
- Optimize before measuring text-heavy fixtures.

## Deliverables

- Glyph cache and positioning policy.
- Text-heavy benchmark deltas.
- Updated text fidelity fixtures.

## Acceptance Criteria

- Repeated text rendering avoids redundant glyph work under bounded memory.
- Text positioning behavior is documented and covered by tests.
- Cache keys do not accidentally mix incompatible glyph outputs.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run text-heavy benchmarks.
- Run text visual comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed in 0087 implementation slice.

- Added a bounded fallback glyph bitmap cache for the built-in ASCII text
  rasterizer.
- Keyed cached glyph bitmaps by normalized character, quantized glyph cell size,
  and mask-only paint policy.
- Preserved user-space subpixel glyph origins through display-list construction
  and final device coverage.
- Documented that color is intentionally outside the cache key because cached
  entries store masks, not painted pixels.
- Added cache key, eviction, and subpixel positioning tests.
- Published the validation report at
  `docs/reports/glyph-cache-subpixel-policy-2026-06-25.md`.
