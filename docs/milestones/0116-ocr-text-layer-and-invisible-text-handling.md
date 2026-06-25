# 0116: OCR Text Layer And Invisible Text Handling

Status: done
Phase: 21
Size: medium
Depends on: 0115

## Goal

Handle scanned PDFs with OCR or invisible text layers without degrading visual
thumbnail fidelity.

## Scope

- Respect text rendering modes that intentionally keep OCR text invisible.
- Ensure invisible text still participates in metadata when extracted.
- Add fixtures with scans, hidden OCR text, and searchable image PDFs.
- Avoid spending raster time on text that cannot affect pixels.

## Non-Goals

- Run OCR.
- Improve OCR accuracy.
- Expose text search APIs in this slice.

## Deliverables

- Invisible text rendering-mode handling.
- OCR-layer fixture coverage.
- Performance report for scanned searchable PDFs.

## Acceptance Criteria

- Hidden OCR layers do not appear in rendered thumbnails.
- Native rendering avoids unnecessary glyph rasterization for invisible text.
- Metadata behavior is documented separately from visual output.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run OCR-layer fixture comparisons.
- Run scanned-document performance benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Commit: `2226900 feat: skip invisible ocr text rasterization`.
- Added generated fixture
  `fixtures/generated/ocr-invisible-text-layer.pdf` and manifest coverage for a
  searchable scan-style PDF with invisible OCR text.
- Added `TextRenderingMode::paints_pixels()` and skipped raster scratch/cache
  work for text modes that cannot paint pixels.
- Added render and native backend tests proving hidden OCR text does not alter
  thumbnail pixels.
- Documented the visual-vs-metadata boundary in `docs/backend/native.md`,
  `docs/policies/document-metadata.md`, and
  `docs/reports/ocr-invisible-text-layer-2026-06-25.md`.
- Validation artifacts:
  - `target/ocr-0116-benchmark.json`: total 104, native rendered 97, fallback
    required 6, errors 1, budget failures 7.
  - `target/ocr-0116-supported-gate.json`: total 45, native rendered 45,
    fallback required 0, errors `{}`.
  - `target/ocr-0116-visual-diff.json`: total 104, exact 34, accepted drift 21,
    blockers 42, native errors 6, PDFium errors 0, both errors 1.
  - OCR fixture visual diff: accepted drift, MAE 0.684, changed ratio 0.027371,
    p95 0, max channel delta 41.
