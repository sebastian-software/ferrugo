# Linearized First Page Loading Report

Date: 2026-06-25.
Milestone: 0114.

## Summary

Milestone 0114 adds a bounded first-page load path for classic-xref PDFs that
declare a valid linearization dictionary. The native backend attempts this path
only for page zero and falls back to the normal full loader whenever hints are
missing, malformed, or not sufficient to render the first page.

This slice does not implement network range fetching or a streaming storage
rewrite. It keeps the existing in-memory input model and uses validated
linearization metadata to reduce the parsed object graph for fast first-page
thumbnail rendering when possible.

## Implementation

- Added `LinearizationDictionary` with `/L`, `/E`, `/O`, `/N`, `/H`, and `/T`
  fields.
- Added `DocumentLoadMetrics` with input bytes, loaded object count, loaded
  object-byte spans, linearization status, and first-page-only status.
- Added `load_linearized_first_page_document` for classic-xref PDFs.
- Added `ClassicDocument::linearized_first_page_tree` for page-zero rendering
  without traversing unloaded later page objects.
- Wired native page-zero rendering to try the bounded loader first and then use
  full classic loading as a safe fallback.
- Added generated fixtures:
  `fixtures/generated/linearized-first-page.pdf` and
  `fixtures/generated/linearized-malformed-hints.pdf`.

The valid generated fixture loads five first-page-section objects instead of
the full seven-object document. The malformed-hints fixture rejects the
first-page path and still renders correctly through the full loader.

## Evidence

Benchmark artifact: `target/linearized-0114-benchmark.json`

- Total: 103 fixtures.
- Native rendered: 96.
- Fallback required: 6.
- Errors: 1 encrypted fixture.
- Budget failures: 7 existing fallback/error cases.

Supported-family gate artifact: `target/linearized-0114-supported-gate.json`

- Total: 45.
- Native rendered: 45.
- Fallback required: 0.
- Families: `browser-print`, `office-export`, `form`.

PDFium visual comparison artifact: `target/linearized-0114-visual-diff.json`

- Total: 103.
- Exact: 34.
- Accepted drift: 20.
- Blockers: 42.
- Native errors: 6.
- PDFium errors: 0.
- Both errors: 1 encrypted fixture.

New fixture results:

| Fixture | Family | Status | MAE | Changed Ratio | p95 | Max Delta | Notes |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| `linearized-first-page.pdf` | `browser-print` | exact | 0.000 | 0.000000 | 0 | 0 | Valid `/E` first-page section renders through the bounded loader. |
| `linearized-malformed-hints.pdf` | `browser-print` | exact | 0.000 | 0.000000 | 0 | 0 | Invalid `/E` hint falls back to the full loader. |

The remaining visual blockers are existing corpus gaps in text/font rendering,
form synthesis, page geometry, color conversion, transparency alpha, and other
renderer areas. The new linearized fixtures are not blockers.

## Validation Commands

- `python3 scripts/generate_fixtures.py`
- `cargo fmt`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p pdfrust-object linearized -- --nocapture`
- `cargo test -p pdfrust-native linearized -- --nocapture`
- `cargo test -p pdfrust-native malformed_linearization -- --nocapture`
- `cargo test -p pdfrust-object`
- `cargo test -p pdfrust-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/linearized-0114-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/linearized-0114-supported-gate.json`
- `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium/out/pdfrust-dylib:/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/linearized-0114-visual-diff.json`

## Follow-Ups

- Add real producer-generated linearized PDFs after privacy review.
- Consider range-fetch abstractions only after the in-memory first-page path is
  stable across real corpus samples.
- Extend first-page inherited page-state recovery if real linearized documents
  frequently keep inherited page metadata outside the `/E` section.
