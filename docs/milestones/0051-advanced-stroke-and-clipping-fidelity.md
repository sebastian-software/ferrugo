# 0051: Advanced Stroke And Clipping Fidelity

Status: todo
Phase: 6
Size: medium
Depends on: 0050

## Goal

Improve vector fidelity for real-world diagrams, charts, and thin-line office
content.

## Scope

- Implement line joins, caps, dash patterns, and miter limits consistently.
- Improve even-odd and nonzero clipping behavior.
- Handle hairlines and near-zero stroke widths with a documented thumbnail
  policy.
- Add reduced fixtures for charts, tables, and vector diagrams.

## Non-Goals

- Full CAD-grade vector precision.
- GPU acceleration.
- Arbitrary precision geometry.

## Deliverables

- Stroke expansion improvements.
- Clipping tests.
- Differential vector fixture comparisons.

## Acceptance Criteria

- Common chart and table lines remain visible and correctly clipped.
- Dash and join behavior is stable across thumbnail sizes.
- Geometry edge cases fail predictably instead of corrupting later drawing.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for vector fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
