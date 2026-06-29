# 0186: Native Text Extraction And Search Parity Gate

Status: done
Phase: 35
Size: medium
Depends on: 0185

## Goal

Establish a native text extraction and search boundary that supports typical
viewer workflows while staying separate from visual rasterization correctness.

## Scope

- Expose text runs, Unicode mapping, page positions, and visibility state where
  the renderer already has reliable information.
- Add search fixtures for office, browser, OCR, and tagged PDFs.
- Document known gaps for complex layout, damaged encodings, and producer bugs.
- Keep extraction memory bounded for large pages and long documents.

## Non-Goals

- Build full semantic document understanding.
- Guarantee exact reading order for malformed or incorrectly tagged files.
- Couple text extraction success to visual render success.

## Deliverables

- Native text extraction API or experimental report.
- Search parity fixture set.
- Policy for visible, invisible, and OCR text layers.

## Acceptance Criteria

- Common searchable PDFs expose stable text and page positions.
- Invisible OCR layers are searchable without affecting raster output.
- Extraction failures are typed and do not panic.

## Validation

- Run native-only `cargo test`.
- Run text extraction fixture tests.
- Run large document extraction memory profiles.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed 2026-06-29.

- Added backend-neutral `TextExtractionBackend`, `TextExtractionOptions`,
  `PageText`, `TextRun`, `PositionedGlyph`, and `TextPoint`.
- Implemented native page text extraction through the existing text display
  list path.
- Added `fixtures/text-extraction-search-manifest.tsv` for visible text,
  office, browser, OCR, and tagged-search baselines.
- Added focused tests for visible text extraction, invisible OCR text
  extraction, and bounded glyph truncation.
- Report: `docs/reports/native-text-extraction-search-boundary-2026-06-29.md`.
