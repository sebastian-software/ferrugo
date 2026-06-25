# 0125: Chart Map And Dashboard Export Coverage

Status: done
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

Completed on 2026-06-25.

- Added four generated fixtures for combo charts with legends, KPI dashboards,
  marker-cluster maps, and heatmap dashboards with translucent overlays.
- Added `fixtures/chart-dashboard-manifest.tsv` with eight focused rows across
  `chart`, `dashboard`, `map`, and `marker-heavy` families.
- Added a native regression test that asserts expected dimensions and visible
  non-background pixels for chart, dashboard, and map fixtures.
- Native fallback gate: 8/8 rendered natively, 0 fallbacks, 0 errors.
- Native benchmark gate: 8/8 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle: 2 exact matches, 6 strict-threshold blockers, 0 native
  render errors, 0 PDFium render errors.
- Report: `docs/reports/chart-dashboard-fidelity-2026-06-25.md`.
