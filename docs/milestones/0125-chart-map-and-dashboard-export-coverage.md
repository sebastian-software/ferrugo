# 0125: Chart Map And Dashboard Export Coverage

Status: todo
Phase: 22
Size: medium
Depends on: 0124

## Goal

Expand native coverage for dashboard exports, charts, and map-like PDFs that
combine labels, legends, patterns, and many small vector marks.

## Scope

- Add fixtures for charts, dashboards, legends, and map-style layouts.
- Cover repeated markers, pattern fills, transparent overlays, and label
  placement.
- Track visual blockers by chart, map, or dashboard subtype.
- Profile rendering cost for many small paths and text runs.

## Non-Goals

- Parse geospatial PDF metadata.
- Reconstruct chart data.
- Implement map projection logic.

## Deliverables

- Chart and dashboard fixture family.
- Map-style rendering gap report.
- Performance profile for many-marker workloads.

## Acceptance Criteria

- Common chart and dashboard exports render with stable visual structure.
- Legends and labels remain legible at thumbnail scale.
- Unsupported geospatial semantics do not block ordinary visual rendering.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run chart and dashboard visual comparisons.
- Run marker-heavy benchmark fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
