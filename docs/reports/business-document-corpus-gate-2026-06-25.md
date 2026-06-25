# Business Document Corpus Gate 2026-06-25

Milestone: 0121.

## Decision

The native renderer now has a focused, committed gate for everyday business
documents: invoices, account statements, receipts, and static business forms.
All seven gated fixtures render natively without PDFium fallback or errors.

PDFium remains useful as a maintainer-only visual oracle. Current visual diffs
show fidelity blockers against PDFium under strict thresholds, but no native or
PDFium render failures.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `business-invoice-dense.pdf` | invoice | logo block, dense totals table, barcode marker, stamp, signature line |
| `account-statement-ledger.pdf` | statement | ledger rows, dense table, barcode marker |
| `thermal-receipt.pdf` | receipt | narrow page, totals, barcode marker |
| `business-form-stamp-signature.pdf` | business form | static form grid, checkbox mark, stamp, barcode marker, signature block |

`fixtures/business-document-manifest.tsv` adds these fixtures plus existing
invoice/statement-style baselines so business-document gates can be sliced by
subtype with `--include-family`.

## Native Gate Evidence

Artifact: `target/business-0121-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `invoice` | 3 | 3 | 0 | 0 |
| `statement` | 2 | 2 | 0 | 0 |
| `receipt` | 1 | 1 | 0 | 0 |
| `business-form` | 1 | 1 | 0 | 0 |
| **Total** | **7** | **7** | **0** | **0** |

## Benchmark Evidence

Artifact: `target/business-0121-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `invoice` | 3 | 3 | 18.054 | 29.812 | 0 |
| `statement` | 2 | 2 | 32.855 | 39.361 | 0 |
| `receipt` | 1 | 1 | 16.740 | 16.740 | 0 |
| `business-form` | 1 | 1 | 45.301 | 45.301 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/business-0121-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `invoice` | 3 | 0 | 0 | 3 | 0 | 0 |
| `statement` | 2 | 0 | 0 | 2 | 0 | 0 |
| `receipt` | 1 | 0 | 0 | 1 | 0 | 0 |
| `business-form` | 1 | 0 | 0 | 1 | 0 | 0 |
| **Total** | **7** | **0** | **0** | **7** | **0** | **0** |

The blocker classification is expected for this phase: native rendering is
functionally present, while strict PDFium visual parity still needs follow-up
work around text metrics, stroke placement, and simple business-form marks.

## Follow-Up Backlog

- Reduce text baseline and glyph metric differences for dense tables.
- Tighten stroke and rectangle placement for small barcode/stamp marks.
- Add image-backed logo and signature reductions once real private examples
  reveal repeatable patterns.
- Keep business-document gates native-only for release readiness; use PDFium
  visual diff as maintainer oracle evidence only.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-cli/src/main.rs crates/pdfrust-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/business-document-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p pdfrust-cli benchmark_config -- --nocapture
cargo test -p pdfrust-native business_document -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/business-document-manifest.tsv --include-family invoice --include-family statement --include-family receipt --include-family business-form --fail-on-fallback --max-edge 160 --output target/business-0121-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/business-document-manifest.tsv --include-family invoice --include-family statement --include-family receipt --include-family business-form --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/business-0121-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/business-document-manifest.tsv --include-family invoice --include-family statement --include-family receipt --include-family business-form --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/business-0121-visual-diff.json
```
