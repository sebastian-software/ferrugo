# 0108: Advanced Shading Mesh Tessellation

Status: todo
Phase: 19
Size: medium
Depends on: 0107

## Goal

Improve native rendering for smooth gradients and mesh shadings found in
presentations, charts, and design-heavy business PDFs.

## Scope

- Implement bounded tessellation for common mesh shading types.
- Reuse gradient sampling buffers across shading patches.
- Add quality knobs tied to thumbnail dimensions.
- Add fixtures for axial, radial, and mesh gradient documents.

## Non-Goals

- Match PDFium at arbitrary zoom levels.
- Add GPU rendering.
- Allow shading tessellation to exceed page memory budgets.

## Deliverables

- Mesh shading tessellation path.
- Performance and visual-diff report.
- Shading quality and budget documentation.

## Acceptance Criteria

- Common gradient PDFs render natively with acceptable drift.
- Tessellation work scales with output resolution, not source complexity alone.
- Oversized shadings are budgeted and reported deterministically.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run shading fixture comparisons.
- Run renderer benchmarks for gradient-heavy PDFs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
