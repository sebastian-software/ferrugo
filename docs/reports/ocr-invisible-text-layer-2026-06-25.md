# OCR Invisible Text Layer Report

Date: 2026-06-25.
Milestone: 0116.

## Summary

The native renderer now treats OCR-style invisible text layers as decoded text
content that must not affect pixels. Text rendering modes that cannot paint
pixels are preserved in the display list but skipped before glyph bitmap lookup,
text scratch expansion, and compositing.

This keeps searchable scan-style PDFs visually faithful while avoiding wasted
raster work for hidden OCR text. Text extraction and search remain outside this
milestone; the metadata policy documents that boundary separately from visual
rendering.

## Implementation

- Added `TextRenderingMode::paints_pixels()`.
- Returned early from `draw_text_run` for `Invisible` and `Clip` text modes.
- Kept display-list text capture intact so decoded invisible text remains
  available to future extraction work.
- Added generated fixture
  `fixtures/generated/ocr-invisible-text-layer.pdf`.
- Added render and native backend tests for hidden OCR text.

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/ocr-0116-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/ocr-0116-supported-gate.json`
- `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium/out/pdfrust-dylib:/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/ocr-0116-visual-diff.json`

## Results

Benchmark summary from `target/ocr-0116-benchmark.json`:

- Total fixtures: 104.
- Native rendered: 97.
- Fallback required: 6.
- Errors: 1.
- Budget failures: 7.

Supported-family gate from `target/ocr-0116-supported-gate.json`:

- Total fixtures: 45.
- Native rendered: 45.
- Fallback required: 0.
- Errors: `{}`.

Visual diff summary from `target/ocr-0116-visual-diff.json`:

- Total fixtures: 104.
- Exact: 34.
- Accepted drift: 21.
- Blockers: 42.
- Native errors: 6.
- PDFium errors: 0.
- Both errors: 1.

OCR fixture result:

- Path: `fixtures/generated/ocr-invisible-text-layer.pdf`.
- Family: `scan`.
- Subsystem: `text-fonts`.
- Status: `accepted_drift`.
- Mean absolute error: 0.684.
- Changed ratio: 0.027371.
- P95 channel delta: 0.
- Max channel delta: 41.
- Native non-white pixels: 18560.
- PDFium non-white pixels: 18560.

## Follow-Ups

- Native text extraction and search parity is tracked separately in milestone
  0186.
- Broader scanner and OCR workflow corpus expansion remains tracked in
  milestone 0147.
