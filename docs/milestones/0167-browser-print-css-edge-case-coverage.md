# 0167: Browser Print CSS Edge Case Coverage

Status: done
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

Completed on 2026-06-26.

Report:

- `docs/reports/browser-print-css-edge-coverage-2026-06-26.md`

Implemented:

- Added four generated browser-print edge fixtures covering repeated/sticky
  header geometry, sibling clipped backgrounds, transformed card geometry, and
  mixed raster/vector print paint order.
- Added `fixtures/browser-print-edge-manifest.tsv` and registered the new
  fixtures in `fixtures/corpus-manifest.tsv`.
- Fixed native clipping scope handling so clipping paths created inside sibling
  `q/Q` graphics-state scopes do not intersect after restore.

Validation:

- `cargo test -p ferrugo-render rasterize_paths_should_restore_clip_with_graphics_state -- --nocapture`
- `cargo test -p ferrugo-native browser_print_edge -- --nocapture`
- Browser-print native support gate: 7 total, 7 native rendered, 0 fallbacks,
  0 errors.
- Browser-print benchmark: 7 total, 7 native rendered, 0 fallbacks, 0 errors,
  0 budget failures.
- Maintainer visual comparison for the four new fixtures: 4 exact, 0 accepted
  drift, 0 blockers, 0 native errors, 0 PDFium errors.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
