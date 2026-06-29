# Tagged PDF Accessibility Metadata Report

Date: 2026-06-25.
Milestone: 0117.

## Summary

The native metadata path now exposes bounded tagged-PDF accessibility signals
without coupling them to thumbnail rendering. The new `accessibility` report
block includes document language, `/MarkInfo /Marked`, RoleMap presence,
structure role count, marked-content reference presence, and traversal
truncation.

Malformed structure trees are reported as metadata `malformed` errors. Rendering
continues to use the page tree and content streams independently, so tagged or
malformed accessibility metadata does not become a rendering prerequisite.

## Implementation

- Added `AccessibilityMetadata` to `ferrugo-thumbnail::DocumentMetadata`.
- Added native metadata extraction for catalog `/Lang`, `/MarkInfo /Marked`,
  `/StructTreeRoot /RoleMap`, structure element roles, and marked-content
  references.
- Bounded structure traversal to 4096 reached values with indirect-object cycle
  protection.
- Added CLI metadata JSON field `accessibility`.
- Added generated fixtures:
  - `fixtures/generated/tagged-accessibility-metadata.pdf`.
  - `fixtures/generated/malformed-tagged-structure.pdf`.

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-native accessibility -- --nocapture`
- `cargo test -p ferrugo-native malformed_tagged -- --nocapture`
- `cargo test -p ferrugo-cli corpus_metadata_json_should_include_manifest_and_page_size -- --nocapture`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/tagged-0117-metadata.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/tagged-0117-supported-gate.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/tagged-0117-benchmark.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/tagged-0117-visual-diff.json`

## Results

Metadata extraction from `target/tagged-0117-metadata.json`:

- Total fixtures: 106.
- `tagged-accessibility-metadata.pdf`: success, `language: "en-US"`,
  `mark_info_marked: true`, RoleMap present, one structure role,
  marked-content reference present, not truncated.
- `malformed-tagged-structure.pdf`: metadata error with class `malformed`.

Supported-family gate from `target/tagged-0117-supported-gate.json`:

- Total fixtures: 46.
- Native rendered: 46.
- Fallback required: 0.
- Errors: `{}`.

Benchmark summary from `target/tagged-0117-benchmark.json`:

- Total fixtures: 106.
- Native rendered: 99.
- Fallback required: 6.
- Errors: 1.
- Budget failures: 7.

Visual diff summary from `target/tagged-0117-visual-diff.json`:

- Total fixtures: 106.
- Exact: 35.
- Accepted drift: 22.
- Blockers: 42.
- Native errors: 6.
- PDFium errors: 0.
- Both errors: 1.

New fixture visual results:

- `malformed-tagged-structure.pdf`: exact, MAE 0.000, changed ratio 0.000000.
- `tagged-accessibility-metadata.pdf`: accepted drift, MAE 0.515, changed ratio
  0.012500, p95 0, max channel delta 129.

## Follow-Ups

- Full reading-order and accessibility tree interpretation remains deferred to
  milestone 0182.
- Accessibility-tagged PDF visual-integrity expansion remains tracked in
  milestone 0154.
