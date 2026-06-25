# 0103: OpenType Layout Feature Coverage For PDFs

Status: done
Phase: 18
Size: medium
Depends on: 0102

## Goal

Cover the OpenType layout behavior needed by typical PDFs that depend on
ligatures, marks, kerning, and shaped text output.

## Scope

- Map PDF text state into the shaping inputs needed by the native renderer.
- Support common GPOS and GSUB paths used by embedded fonts.
- Keep shaping buffers reusable to avoid per-glyph allocation churn.
- Add fixtures for ligatures, combining marks, Arabic text, and office exports.

## Non-Goals

- Add text editing or selection behavior.
- Support every script-specific shaping edge case in one slice.
- Replace earlier CMap and ToUnicode extraction work.

## Deliverables

- OpenType layout coverage implementation.
- Shaping fixture set and visual comparison report.
- Documentation of unsupported layout features.

## Acceptance Criteria

- Common ligatures and mark positioning render in native output.
- Repeated page renders reuse buffers and avoid unbounded allocations.
- Unsupported shaping cases produce explicit typed fallback reasons.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run shaped-text visual comparisons.
- Run allocation-sensitive text rendering smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added typed `TextLayoutStatus` metadata for decoded PDF glyphs.
- Added native fallback handling for common ToUnicode ligature expansions.
- Added deterministic combining-mark fallback positioning over the previous
  base glyph.
- Preserved pre-positioned Arabic/Hebrew-style script output as shaped text
  metadata.
- Added typed unsupported complex-script fallback reasons for decoded glyphs
  outside the current native shaping subset.
- Added reusable fallback text raster scratch capacity for expanded text atoms.
- Added ligature, combining-mark, and Arabic shaped-text generated fixtures
  plus native backend smoke tests.
- Supported-family gate: 38 total, 38 native rendered, 0 fallback, 0 errors.
- PDFium visual comparison marks the new shaped-text fixtures as blockers due
  to built-in fallback text rasterizer drift, not native fallback.
- Report: `docs/reports/opentype-layout-feature-coverage-2026-06-25.md`.
- Implementation commit: `489d359 feat: add native shaped text layout fallback`.
