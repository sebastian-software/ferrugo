# Scanner And OCR Workflow Corpus

Date: 2026-06-26.
Milestone: 0147.

## Summary

The scanner/OCR workflow corpus now has a focused manifest at
`fixtures/scanner-ocr-workflow-manifest.tsv`. It separates supported scanner
workflows from the intentionally unsupported scanner-codec backlog.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `scanner-skewed-mailroom-page.pdf` | Skewed mailroom scan with Flate-compressed DeviceGray image data. |
| `scanner-large-image-budget.pdf` | Large compressed scan image for decode and thumbnail memory-budget coverage. |
| `scanner-ocr-form-overlay.pdf` | Scan-plus-form page with invisible OCR text and visible form overlay lines. |

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family skew --include-family large-image --include-family form-overlay --fail-on-fallback --max-edge 160 --output target/scanner-0147-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 10 | 10 | 0 | 0 |

Supported family result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `compression` | 3 | 3 | 0 | 0 |
| `crop` | 1 | 1 | 0 | 0 |
| `form-overlay` | 1 | 1 | 0 | 0 |
| `large-image` | 1 | 1 | 0 | 0 |
| `ocr-layer` | 2 | 2 | 0 | 0 |
| `rotation` | 1 | 1 | 0 | 0 |
| `skew` | 1 | 1 | 0 | 0 |

## Unsupported Codec Backlog

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family unsupported-filter --max-edge 160 --output target/scanner-0147-unsupported-backlog.json
```

Result:

| Family | Total | Native rendered | Fallback required | Fallback category |
| --- | ---: | ---: | ---: | --- |
| `unsupported-filter` | 3 | 0 | 3 | `image.filter` |

## Memory And Decode Budget

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family skew --include-family large-image --include-family form-overlay --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/scanner-0147-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `compression` | 3 | 10.918 | 29.696 | 0 |
| `crop` | 1 | 45.826 | 45.826 | 0 |
| `form-overlay` | 1 | 24.311 | 24.311 | 0 |
| `large-image` | 1 | 33.527 | 33.527 | 0 |
| `ocr-layer` | 2 | 10.273 | 18.100 | 0 |
| `rotation` | 1 | 3.273 | 3.273 | 0 |
| `skew` | 1 | 22.170 | 22.170 | 0 |

## Visual Oracle

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family skew --include-family large-image --include-family form-overlay --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/scanner-0147-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 3 | 1 | 6 | 0 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `scanner-skewed-mailroom-page.pdf` | blocker | `page-geometry` | 5.095 | 13 | 0.764805 |
| `scanner-large-image-budget.pdf` | blocker | `images-color` | 2.079 | 8 | 0.239278 |
| `scanner-ocr-form-overlay.pdf` | blocker | `rendering-core` | 2.486 | 13 | 0.608717 |

These blockers are fidelity deltas, not native runtime fallbacks. They route to
scan image resampling, page-transform/skew parity, and visible overlay
composition.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `scanner-skewed-mailroom-page.pdf` | 1,332 |
| `scanner-large-image-budget.pdf` | 5,407 |
| `scanner-ocr-form-overlay.pdf` | 1,591 |
| **Total new PDF bytes** | **8,330** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- New fixture content is synthetic and has no private or customer document
  references.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family skew --include-family large-image --include-family form-overlay --fail-on-fallback --max-edge 160 --output target/scanner-0147-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family unsupported-filter --max-edge 160 --output target/scanner-0147-unsupported-backlog.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family skew --include-family large-image --include-family form-overlay --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/scanner-0147-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family skew --include-family large-image --include-family form-overlay --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/scanner-0147-visual-diff.json
cargo test -p ferrugo-native native_backend_should_bound_adversarial_huge_image_dimensions -- --nocapture
cargo test -p ferrugo-render image_resources_should_enforce -- --nocapture
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/scanner-skewed-mailroom-page.pdf fixtures/generated/scanner-large-image-budget.pdf fixtures/generated/scanner-ocr-form-overlay.pdf
```
