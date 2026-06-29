# 0184: Print Shop Imposition And Booklet PDF Coverage

Status: done
Phase: 34
Size: medium
Depends on: 0183

## Goal

Cover typical booklet, n-up, crop-mark, bleed, and imposed PDFs without making
the renderer depend on PDFium for print-preview workflows.

## Scope

- Add fixtures for imposed pages, crop marks, bleed boxes, booklet spreads, and
  rotated content.
- Verify page box selection, transform composition, and clipping behavior.
- Document prepress features that remain outside the typical-document boundary.
- Measure high-DPI thumbnail output for imposed pages.

## Non-Goals

- Build a print imposition tool.
- Implement full prepress proofing or color-accurate output.
- Treat rare print-shop operators as default support requirements.

## Deliverables

- Print imposition coverage report.
- Page geometry and clipping regression fixtures.
- Updated support matrix entries for prepress-adjacent cases.

## Acceptance Criteria

- Common booklet and n-up PDFs render with correct geometry.
- Crop and bleed handling is deterministic and documented.
- Unsupported print-production cases remain typed.

## Validation

- Run native-only `cargo test`.
- Run page geometry fixture comparisons.
- Run high-DPI thumbnail visual checks.
- Run benchmark profiles for imposed pages.

## Completion Notes

Completed 2026-06-29.

- Added `print-booklet-spread.pdf` and `print-nup-imposed-sheet.pdf` plus
  `fixtures/print-imposition-manifest.tsv`.
- Added native regression coverage for imposed sheet geometry:
  `print-booklet-spread.pdf` renders to `460 x 280` from CropBox and
  `print-nup-imposed-sheet.pdf` renders to `420 x 300` from MediaBox.
- Updated the Poppler visual oracle to use `pdftoppm -cropbox`, matching the
  native thumbnail page-box policy for page-box-sensitive fixtures.
- Native supported gate: 4/4 native rendered, 0 fallbacks, 0 errors.
- Benchmark gate: 4/4 native rendered, 0 errors, 0 budget failures.
- Poppler visual gate for the two new fixtures: 2 accepted drift, 0 blockers,
  0 native/reference errors.
- Report: `docs/reports/print-imposition-booklet-coverage-2026-06-29.md`.
