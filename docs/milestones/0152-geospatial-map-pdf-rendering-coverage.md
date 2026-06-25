# 0152: Geospatial Map PDF Rendering Coverage

Status: todo
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

Empty until done.
