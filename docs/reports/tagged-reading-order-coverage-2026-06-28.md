# Tagged Reading Order Coverage 2026-06-28

Milestone: 0182.
Status: in progress.

## Summary

The native metadata path now exposes bounded reading-order signals for tagged
PDFs without turning accessibility metadata into a visual-rendering
prerequisite or a full accessibility API.

New metadata fields:

- `marked_content_reference_count`
- `page_content_reference_count`
- `alt_text_count`
- `reading_order_warning_count`

The counts are derived while traversing `/StructTreeRoot /K` with the existing
metadata traversal budget. MCID/MCR references inherit page context from parent
structure elements, so common producer output can be checked for page-associated
reading-order entries without retaining PDF bytes or content operands.

## Fixture Additions

| Fixture | Family | Purpose | Bytes |
| --- | --- | --- | ---: |
| `tagged-invoice-reading-order.pdf` | `tagged-invoice` | Tagged invoice with header, table, and total entries associated with page content. | 1,713 |
| `tagged-reading-order-missing-page-context.pdf` | `reading-order-warning` | Boundary fixture where a marked-content reference lacks page context and produces one warning. | 1,012 |

The focused tagged manifest now covers report, form, office, invoice,
structure-heavy, metadata-baseline, and warning-boundary cases.

## Metadata Evidence

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- extract-corpus-metadata fixtures/generated \
  --manifest fixtures/tagged-pdf-visual-manifest.tsv \
  --output target/tagged-0182-metadata.json
```

Focused results:

| Fixture | Roles | MC refs | Page refs | Alt text | Warnings | Truncated |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `tagged-accessibility-metadata.pdf` | 1 | 1 | 1 | 0 | 0 | false |
| `tagged-form-visual-integrity.pdf` | 2 | 1 | 1 | 0 | 0 | false |
| `tagged-invoice-reading-order.pdf` | 4 | 3 | 3 | 0 | 0 | false |
| `tagged-office-alt-text.pdf` | 3 | 2 | 2 | 1 | 0 | false |
| `tagged-reading-order-missing-page-context.pdf` | 2 | 1 | 0 | 0 | 1 | false |
| `tagged-report-visual-integrity.pdf` | 3 | 2 | 2 | 0 | 0 | false |
| `tagged-structure-heavy-report.pdf` | 65 | 64 | 64 | 0 | 0 | false |

## Native Gates

Supported render gate:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/tagged-pdf-visual-manifest.tsv \
  --include-family tagged-report \
  --include-family tagged-form \
  --include-family tagged-office \
  --include-family tagged-invoice \
  --include-family reading-order-warning \
  --include-family structure-heavy \
  --include-family metadata-baseline \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/tagged-0182-supported-gate.json
```

Result: 7 total, 7 native rendered, 0 fallback required, 0 errors.

Benchmark gate:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/tagged-pdf-visual-manifest.tsv \
  --include-family tagged-report \
  --include-family tagged-form \
  --include-family tagged-office \
  --include-family tagged-invoice \
  --include-family reading-order-warning \
  --include-family structure-heavy \
  --include-family metadata-baseline \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/tagged-0182-benchmark.json
```

Result: 7 total, 7 native rendered, 0 fallback required, 0 errors, 0 budget
failures.

## Visual Oracle Status

The tagged visual-diff command was executed, but the local PDFium oracle library
was not available at
`/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib`.
The generated artifact therefore reports 7 PDFium errors and no native errors.

Command attempted:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/tagged-pdf-visual-manifest.tsv \
  --include-family tagged-report \
  --include-family tagged-form \
  --include-family tagged-office \
  --include-family tagged-invoice \
  --include-family reading-order-warning \
  --include-family structure-heavy \
  --include-family metadata-baseline \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/tagged-0182-visual-diff.json
```

This keeps 0182 in progress until a maintainer visual-oracle run is available.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo fmt
cargo test -p pdfrust-native tagged_visual -- --nocapture
cargo test -p pdfrust-native reading_order -- --nocapture
cargo test -p pdfrust-native accessibility -- --nocapture
cargo test -p pdfrust-cli corpus_metadata_json_should_include_manifest_and_page_size -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family tagged-invoice --include-family reading-order-warning --include-family structure-heavy --include-family metadata-baseline --fail-on-fallback --max-edge 160 --output target/tagged-0182-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --output target/tagged-0182-metadata.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family tagged-invoice --include-family reading-order-warning --include-family structure-heavy --include-family metadata-baseline --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/tagged-0182-benchmark.json
wc -c fixtures/generated/tagged-invoice-reading-order.pdf fixtures/generated/tagged-reading-order-missing-page-context.pdf
find fixtures/generated -name '*.pdf' -size +512k -print
```
