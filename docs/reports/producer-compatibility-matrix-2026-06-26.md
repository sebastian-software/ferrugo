# Producer Compatibility Matrix 2026-06-26

Milestone: 0163.

## Decision

Track producer compatibility through privacy-safe synthetic reductions first.
The new matrix records producer, version style, document family, workflow, and
feature pressure without claiming complete coverage for any real producer.

The CLI-compatible gate manifest is:
`fixtures/producer-compatibility-manifest.tsv`.

The expanded producer metadata matrix is:
`fixtures/producer-compatibility-matrix.tsv`.

## Matrix Scope

Supported producer-style rows:

| Producer group | Rows | Native status | Main feature pressure |
| --- | ---: | --- | --- |
| Office suites | 3 | supported | headers/footers, spreadsheets, charts, comments, handouts. |
| Browsers | 3 | supported | CSS backgrounds, tables, clipping, forms, barcode-like content. |
| Scanners | 2 | supported | skewed image-like content and mixed Flate/DCT compression. |
| Accounting/banking | 2 | supported | dense tables, ledgers, barcode markers, logos, stamps. |
| Government forms | 2 | supported | AcroForm widgets, stamps, barcode markers, strict tables. |
| PDF 2.0 producer baseline | 1 | supported | `%PDF-2.0` and catalog `/Version /2.0` with existing native features. |

Typed unsupported producer-style boundaries:

| Producer group | Fixture | Bucket | Owner route |
| --- | --- | --- | --- |
| Layered presentation export | `optional-content-ocmd.pdf` | `graphics.optional-content` | 0192 optional-content policy. |
| Fax/scanner export | `unsupported-ccitt-image.pdf` | `image.filter` | 0209 codec deployment policy. |

Email-client and design-tool producer reductions are still gaps. They should be
added in later corpus milestones rather than represented by unrelated samples.

## Gate Evidence

Supported producer gate artifact:
`target/producer-0163-supported-gate.json`.

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `accounting` | 2 | 2 | 0 | 0 |
| `browser` | 3 | 3 | 0 | 0 |
| `government` | 2 | 2 | 0 | 0 |
| `office-suite` | 3 | 3 | 0 | 0 |
| `pdf20` | 1 | 1 | 0 | 0 |
| `scanner` | 2 | 2 | 0 | 0 |
| **Total** | **13** | **13** | **0** | **0** |

Unsupported producer classification artifact:
`target/producer-0163-classification.json`.

| Family | Total | Native rendered | Fallback required | Buckets |
| --- | ---: | ---: | ---: | --- |
| `unsupported-boundary` | 2 | 0 | 2 | `graphics.optional-content`, `image.filter` |

Visual comparison artifact:
`target/producer-0163-visual-diff.json`.

The visual subset reports 13/13 blockers with 0 native errors and 0 PDFium
errors. Subsystem routing is 12 `rendering-core` blockers and 1
`page-geometry` blocker. This is triage evidence, not a runtime support failure
for this matrix milestone. It identifies fidelity work for the owner routes
listed in the expanded matrix.

Benchmark artifact:
`target/producer-0163-benchmark.json`.

The supported producer subset renders natively with 0 fallbacks, 0 errors, and
0 benchmark budget failures. Mean render times by family are currently between
25.559 ms (`scanner`) and 44.901 ms (`government`) at `max_edge = 160`.

## Follow-Up Slices

1. Office and accounting rows should feed 0166 and 0204 because their main
   risks are dense tables, charts, thin strokes, and vector effects.
2. Browser rows should feed 0167 because they stress print CSS reductions:
   clipping, backgrounds, form-like print content, and annotation links.
3. Scanner rows should feed 0170 and 0209 because they split supported
   image-heavy memory behavior from unsupported fax/specialized codecs.
4. Government rows should feed 0206 because visual form appearance and
   flattening are the likely next product-facing gaps.
5. PDF 2.0 producer rows should feed 0181 after broader PDF 2.0 usage evidence
   exists.

## Validation

Commands run:

```sh
node --input-type=module -e 'import fs from "node:fs"; const rows = fs.readFileSync("fixtures/producer-compatibility-matrix.tsv", "utf8").trimEnd().split("\n").map((line) => line.split("\t")); if (rows[0].length !== 10 || rows.slice(1).some((row) => row.length !== 10)) throw new Error("producer matrix must have 10 TSV columns"); console.log(`${rows.length - 1} producer matrix rows validated`);'
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/producer-compatibility-manifest.tsv --include-family office-suite --include-family browser --include-family scanner --include-family accounting --include-family government --include-family pdf20 --fail-on-fallback --max-edge 160 --output target/producer-0163-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/producer-compatibility-manifest.tsv --include-family unsupported-boundary --max-edge 160 --output target/producer-0163-classification.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/producer-compatibility-manifest.tsv --include-family office-suite --include-family browser --include-family scanner --include-family accounting --include-family government --include-family pdf20 --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/producer-0163-visual-diff.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/producer-compatibility-manifest.tsv --include-family office-suite --include-family browser --include-family scanner --include-family accounting --include-family government --include-family pdf20 --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/producer-0163-benchmark.json
cargo fmt --check
```
