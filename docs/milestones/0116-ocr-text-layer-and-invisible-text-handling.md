# 0116: OCR Text Layer And Invisible Text Handling

Status: in-progress
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

Empty until done.
