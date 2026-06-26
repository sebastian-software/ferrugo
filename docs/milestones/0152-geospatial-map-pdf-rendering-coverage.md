# 0152: Geospatial Map PDF Rendering Coverage

Status: done
Phase: 28
Size: medium
Depends on: 0151

## Goal

Cover map-like PDFs with dense vectors, labels, tiles, transparency, spot colors,
and large page coordinate systems.

## Scope

- Add public or synthetic map fixtures with labels, legends, grids, raster tiles,
  vector layers, and transparent overlays.
- Track optional content, pattern, image, and path interactions.
- Measure render time and memory for dense map pages.
- Document unsupported geospatial metadata behavior separately from visuals.

## Non-Goals

- Implement GIS coordinate queries.
- Provide map layer UI.
- Guarantee semantic geospatial extraction.

## Deliverables

- Map PDF corpus entries.
- Dense vector and layer interaction report.
- Follow-up backlog for map-specific renderer gaps.

## Acceptance Criteria

- Common map pages render natively with readable labels and visible layers.
- Optional content policy is explicit and deterministic.
- Dense pages remain within performance and memory budgets.

## Validation

- Run map-family visual comparison.
- Run optional-content regression tests.
- Run benchmark subset for dense map pages.
- Run native-only supported corpus gate.

## Completion Notes

Completed on 2026-06-26.

- Added three synthetic map fixtures:
  `map-raster-tile-routes.pdf`, `map-transparent-zoning-overlay.pdf`, and
  `map-optional-layer-policy.pdf`.
- Added `fixtures/map-rendering-manifest.tsv` to separate supported visual map
  features from the OCMD optional-content unsupported boundary.
- Extended the native chart/dashboard smoke test to compile and render the new
  map fixtures.
- Native supported gate is green at 7/7 rendered, 0 fallbacks, and 0 errors.
- OCMD remains explicitly typed as `graphics.optional-content`.
- Benchmark reports 0 budget failures.
- Visual oracle reports 2 exact rows, 5 fidelity blockers, and 0 native/PDFium
  errors; those deltas route to raster tiles, alpha overlays, labels, and map
  route/grid rendering parity.
- Report: `docs/reports/geospatial-map-rendering-coverage-2026-06-26.md`.
