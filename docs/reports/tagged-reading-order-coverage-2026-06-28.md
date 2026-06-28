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

That initial PDFium attempt kept 0182 in progress until another maintainer
visual-oracle path was available.

PDFium-free maintainer oracle tooling was added with `visual-diff-poppler` and
run against the same tagged fixture slice. Poppler rendered all 7 fixtures with
0 reference errors, and the native renderer produced 0 render errors. Follow-up
work refined standard-base-font fallback masks, added case-sensitive lowercase
x-height masks, aligned Poppler reference renders to the native target
dimensions, and added coverage-aware antialiasing for text fallback rectangles.
That removed the former 1px dimension mismatches from the oracle report,
improved text-heavy drift metrics, and moved the reading-order warning fixture
to accepted drift. The strict threshold report still classifies 6 fixtures as
blockers, so 0182 remains in progress for visual-fidelity work rather than
oracle availability.

Poppler command:

```sh
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/tagged-pdf-visual-manifest.tsv \
  --include-family tagged-report \
  --include-family tagged-form \
  --include-family tagged-office \
  --include-family tagged-invoice \
  --include-family reading-order-warning \
  --include-family structure-heavy \
  --include-family metadata-baseline \
  --max-edge 160 \
  --timeout 20 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/tagged-0182-poppler-visual-diff.json
```

Result: 7 total, 0 exact, 1 accepted drift, 6 blockers, 0 native errors,
0 reference errors, 0 both errors.

Blocker split:

| Fixture | Status | Evidence |
| --- | --- | --- |
| `tagged-accessibility-metadata.pdf` | blocker | MAE 1.228, p95 1, changed ratio 0.161765, max delta 171. |
| `tagged-form-visual-integrity.pdf` | blocker | MAE 4.692, p95 28, changed ratio 0.085689, max delta 255. |
| `tagged-invoice-reading-order.pdf` | blocker | MAE 10.972, p95 97, changed ratio 0.138462, max delta 209. |
| `tagged-office-alt-text.pdf` | blocker | MAE 13.313, p95 105, changed ratio 0.110748, max delta 209. |
| `tagged-reading-order-missing-page-context.pdf` | accepted drift | MAE 1.913, p95 0, changed ratio 0.029885, max delta 217. |
| `tagged-report-visual-integrity.pdf` | blocker | MAE 12.050, p95 100, changed ratio 0.135776, max delta 201. |
| `tagged-structure-heavy-report.pdf` | blocker | MAE 15.210, p95 102, changed ratio 0.222650, max delta 204. |

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
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/tagged-pdf-visual-manifest.tsv --include-family tagged-report --include-family tagged-form --include-family tagged-office --include-family tagged-invoice --include-family reading-order-warning --include-family structure-heavy --include-family metadata-baseline --max-edge 160 --timeout 20 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/tagged-0182-poppler-visual-diff.json
wc -c fixtures/generated/tagged-invoice-reading-order.pdf fixtures/generated/tagged-reading-order-missing-page-context.pdf
find fixtures/generated -name '*.pdf' -size +512k -print
```
