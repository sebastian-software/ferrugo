# E-Signature Workflow Document Coverage

Date: 2026-06-26.
Milestone: 0153.

## Summary

The e-signature workflow corpus now has a focused manifest at
`fixtures/e-signature-workflow-manifest.tsv`. It covers static visual contract
workflow surfaces while keeping cryptographic validation out of the renderer
boundary.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `e-signature-contract-workflow.pdf` | Contract-style page with signer initials, date field text, signature widget appearance, stamp appearance, and `/ByteRange` signature metadata. |
| `e-signature-audit-certificate.pdf` | Completion certificate with audit event table, stamp-like seal, and QR-style marker. |
| `e-signature-incremental-revision.pdf` | Incrementally updated signed revision that proves the latest page/catalog revision remains renderable. |

Existing signature baselines remain in the focused manifest:
`digital-signature-appearance.pdf` and
`acroform-signature-placeholder.pdf`.

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/e-signature-workflow-manifest.tsv --include-family contract-workflow --include-family audit-trail --include-family incremental-signature --include-family signature-appearance --fail-on-fallback --max-edge 160 --output target/e-signature-0153-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 5 | 5 | 0 | 0 |

Family result:

| Family | Total | Native rendered | Fallback required |
| --- | ---: | ---: | ---: |
| `audit-trail` | 1 | 1 | 0 |
| `contract-workflow` | 1 | 1 | 0 |
| `incremental-signature` | 1 | 1 | 0 |
| `signature-appearance` | 2 | 2 | 0 |

## Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/e-signature-workflow-manifest.tsv --include-family contract-workflow --include-family audit-trail --include-family incremental-signature --include-family signature-appearance --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/e-signature-0153-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `audit-trail` | 1 | 36.037 | 36.037 | 0 |
| `contract-workflow` | 1 | 46.374 | 46.374 | 0 |
| `incremental-signature` | 1 | 35.169 | 35.169 | 0 |
| `signature-appearance` | 2 | 25.254 | 35.783 | 0 |

## Visual Oracle

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/e-signature-workflow-manifest.tsv --include-family contract-workflow --include-family audit-trail --include-family incremental-signature --include-family signature-appearance --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/e-signature-0153-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 0 | 1 | 4 | 0 | 0 |

Subsystem result:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `annotations-forms` | 5 | 1 | 4 | 0 |

Fixture classifications:

| Fixture | Status | MAE | p95 | Changed ratio | Native non-white | PDFium non-white |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `digital-signature-appearance.pdf` | accepted drift | 0.137 | 0 | 0.011944 | 3,000 | 3,000 |
| `acroform-signature-placeholder.pdf` | blocker | 9.437 | 15 | 0.208333 | 3,000 | 0 |
| `e-signature-audit-certificate.pdf` | blocker | 8.528 | 42 | 0.153728 | 18,240 | 18,240 |
| `e-signature-contract-workflow.pdf` | blocker | 10.226 | 66 | 0.192672 | 4,706 | 5,150 |
| `e-signature-incremental-revision.pdf` | blocker | 7.632 | 23 | 0.132617 | 1,580 | 1,320 |

These blockers are fidelity deltas in static annotation/form rendering. They do
not require PDFium fallback for native server-side rendering.

## Validation Boundary

Signature support remains visual and metadata-only:

- Existing signature appearance streams render through the static annotation and
  AcroForm paths.
- `DocumentStructure::has_signature_fields` and
  `DocumentStructure::has_signature_byte_range` report presence only.
- The renderer does not validate certificates, trust chains, digest contents,
  timestamps, revocation, or legal signature status.
- JavaScript and dynamic form execution remain out of scope.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `e-signature-contract-workflow.pdf` | 2,268 |
| `e-signature-audit-certificate.pdf` | 1,570 |
| `e-signature-incremental-revision.pdf` | 2,070 |
| **Total new PDF bytes** | **5,908** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- New fixture content is synthetic and contains no real contracts, signers,
  certificate material, or audit records.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo test -p ferrugo-native e_signature -- --nocapture
cargo test -p ferrugo-native signature_presence -- --nocapture
cargo test -p ferrugo-native annotation_appearance -- --nocapture
cargo test -p ferrugo-native incremental -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/e-signature-workflow-manifest.tsv --include-family contract-workflow --include-family audit-trail --include-family incremental-signature --include-family signature-appearance --fail-on-fallback --max-edge 160 --output target/e-signature-0153-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/e-signature-workflow-manifest.tsv --include-family contract-workflow --include-family audit-trail --include-family incremental-signature --include-family signature-appearance --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/e-signature-0153-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/e-signature-workflow-manifest.tsv --include-family contract-workflow --include-family audit-trail --include-family incremental-signature --include-family signature-appearance --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/e-signature-0153-visual-diff.json
wc -c fixtures/generated/e-signature-contract-workflow.pdf fixtures/generated/e-signature-audit-certificate.pdf fixtures/generated/e-signature-incremental-revision.pdf
find fixtures/generated -name '*.pdf' -size +512k -print
```
