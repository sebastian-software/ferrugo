# Embedded Files Portfolio Report

Date: 2026-06-25.
Milestone: 0113.

## Summary

Milestone 0113 adds inert visibility for embedded files, PDF portfolios, and
file-attachment annotations. The native renderer keeps attachments out of the
thumbnail execution path: it reports presence-only metadata and renders visible
attachment annotations only when a normal appearance stream is present.

The implementation intentionally does not extract, open, execute, preview, or
sort embedded payloads. That keeps the thumbnail and metadata APIs bounded and
side-effect free while allowing callers to classify these documents without a
PDFium runtime.

## Implementation

- Added `DocumentStructure::has_embedded_files`.
- Added `DocumentStructure::has_portfolio_collection`.
- Added `DocumentStructure::has_file_attachment_annotations`.
- Added CLI metadata JSON fields for the new structure signals.
- Added bounded native metadata scanning for catalog `/Names /EmbeddedFiles`,
  catalog `/Collection`, and page `/Subtype /FileAttachment` annotations.
- Added generated fixtures:
  `fixtures/generated/embedded-source-file.pdf`,
  `fixtures/generated/portfolio-embedded-files.pdf`, and
  `fixtures/generated/file-attachment-annotation.pdf`.

The annotation metadata scan is bounded to 4096 annotation entries. Embedded
payload bytes remain inert fixture data and are never decoded by the renderer.

## Evidence

Benchmark artifact: `target/embedded-0113-benchmark.json`

- Total: 101 fixtures.
- Native rendered: 94.
- Fallback required: 6.
- Errors: 1 encrypted fixture.
- Budget failures: 7 existing fallback/error cases.

Supported-family gate artifact: `target/embedded-0113-supported-gate.json`

- Total: 43.
- Native rendered: 43.
- Fallback required: 0.
- Families: `browser-print`, `office-export`, `form`.

PDFium visual comparison artifact: `target/embedded-0113-visual-diff.json`

- Total: 101.
- Exact: 32.
- Accepted drift: 20.
- Blockers: 42.
- Native errors: 6.
- PDFium errors: 0.
- Both errors: 1 encrypted fixture.

New fixture results:

| Fixture | Family | Status | MAE | Changed Ratio | p95 | Max Delta | Notes |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| `embedded-source-file.pdf` | `mixed-layout` | accepted drift | 1.446 | 0.044444 | 0 | 64 | Native and PDFium both render the source attachment page without opening the payload. |
| `file-attachment-annotation.pdf` | `mixed-layout` | accepted drift | 0.008 | 0.000185 | 0 | 54 | The visible attachment appearance renders through the normal annotation path. |
| `portfolio-embedded-files.pdf` | `mixed-layout` | exact | 0.000 | 0.000000 | 0 | 0 | Portfolio catalog metadata does not alter page rendering. |

The remaining visual blockers are existing corpus gaps in text/font rendering,
form synthesis, page geometry, color conversion, transparency alpha, and other
renderer areas. The new embedded-file and portfolio fixtures are not blockers.

## Validation Commands

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-native attachment -- --nocapture`
- `cargo test -p ferrugo-native portfolio -- --nocapture`
- `cargo test -p ferrugo-native embedded -- --nocapture`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/embedded-0113-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/embedded-0113-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/embedded-0113-visual-diff.json`

## Follow-Ups

- Keep attachment extraction out of thumbnail rendering unless a separate,
  explicit API is designed.
- Let later corpus milestones expand email-export and document-management
  portfolio samples with real privacy-reviewed fixtures.
- Preserve the presence-only semantics in metadata so callers do not confuse
  inert classification with attachment validation or safety scanning.
