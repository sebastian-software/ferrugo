# 0167: Browser Print CSS Edge Case Coverage

Status: todo
Phase: 31
Size: medium
Depends on: 0166

## Goal

Cover browser-generated PDFs that use common CSS print features and expose
renderer gaps in clipping, transforms, images, and transparency.

## Scope

- Add fixtures generated from common browser print workflows.
- Include sticky headers, repeated table headers, transformed elements, shadows,
  clipped backgrounds, and mixed raster/vector content.
- Fix native renderer gaps that are narrow and measurable.
- Add typed unsupported coverage for cases outside the renderer boundary.

## Non-Goals

- Implement a CSS engine.
- Validate HTML layout before PDF generation.
- Support browser-specific private metadata.

## Deliverables

- Browser print edge-case fixture set.
- Native renderer fixes for accepted edge cases.
- Browser-print coverage report.

## Acceptance Criteria

- Common browser print edge-case PDFs render within visual thresholds.
- Unsupported cases fail with typed reasons.
- Fixture generation steps are reproducible.

## Validation

- Run native-only `cargo test`.
- Run browser-print corpus gate.
- Run visual comparison for new fixtures.
- Run benchmark summary for browser-print samples.

## Completion Notes

Empty until done.
