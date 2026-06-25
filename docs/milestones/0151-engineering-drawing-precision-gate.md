# 0151: Engineering Drawing Precision Gate

Status: todo
Phase: 28
Size: medium
Depends on: 0150

## Goal

Improve native renderer precision for CAD exports, engineering drawings,
floorplans, schematics, and other line-heavy technical PDFs.

## Scope

- Add fixtures for thin strokes, dashed lines, clipped details, transforms,
  symbols, labels, and large page sizes.
- Measure path, stroke, clipping, and transform drift separately.
- Add high-zoom or region-based checks for precision-sensitive pages.
- Track performance for large vector command streams.

## Non-Goals

- Implement CAD semantics.
- Support infinite zoom or viewer interaction.
- Optimize every rare path operator before common drawings pass.

## Deliverables

- Engineering drawing corpus entries.
- Precision-focused visual report.
- Backlog for path, stroke, clipping, and transform fixes.

## Acceptance Criteria

- Common drawings render natively without missing major geometry.
- Thin strokes and dashed lines remain visible and stable.
- Large vector pages stay within documented time and memory budgets.

## Validation

- Run engineering-family visual comparison.
- Run path and stroke regression tests.
- Run benchmark subset for vector-heavy pages.
- Run native-only supported corpus gate.

## Completion Notes

Empty until done.
