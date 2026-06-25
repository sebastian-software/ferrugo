# 0184: Print Shop Imposition And Booklet PDF Coverage

Status: todo
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

Empty until done.
