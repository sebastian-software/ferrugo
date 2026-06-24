# 0072: Vector Graphics Stress Coverage

Status: todo
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

Empty until done.
