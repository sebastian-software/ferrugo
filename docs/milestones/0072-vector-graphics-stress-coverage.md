# 0072: Vector Graphics Stress Coverage

Status: done
Phase: 11
Size: medium
Depends on: 0071

## Goal

Make vector-heavy diagrams, charts, and technical PDFs render reliably.

## Scope

- Expand path flattening, clipping, dash, join, cap, and winding-rule coverage.
- Add stress fixtures for many small paths and deeply nested clips.
- Measure rasterization time and allocation behavior.
- Keep fallback thresholds explicit for pathological vector content.

## Non-Goals

- CAD-grade rendering precision.
- GPU acceleration.
- Infinite path or clip complexity.

## Deliverables

- Vector stress fixture set.
- Rasterizer improvements for common chart and diagram constructs.
- Benchmarks for path-heavy pages.

## Acceptance Criteria

- Representative chart and diagram PDFs render natively.
- Path-heavy pages stay within documented time and memory budgets.
- Excessive vector complexity fails predictably with budget diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run vector stress corpus comparisons.
- Run targeted raster benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `test: add vector stress coverage` implementation and the
`docs: complete vector stress coverage` report update.

- Added `fixtures/generated/vector-stress.pdf`, a generated chart-like vector
  stress fixture with nested clips, many small path items, and a cubic curve.
- Added render-crate and native-backend tests for native rasterization of the
  fixture.
- Added a targeted flattened-segment budget test that fails predictably with
  `PathComplexityOverflow`.
- Recorded vector corpus comparison, native/PDFium render evidence, and timing
  notes in `docs/reports/vector-stress-coverage-2026-06-24.md`.
