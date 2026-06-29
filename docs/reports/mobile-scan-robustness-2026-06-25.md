# Mobile Scan Robustness 2026-06-25

Milestone: 0129.

## Decision

Mobile scanner and camera-app PDFs now have a focused native gate. The native
renderer renders all nine supported mobile-scan manifest rows without PDFium
fallback, errors, or benchmark budget failures.

The milestone also records the scanner-style unsupported image filter backlog:
CCITT, JBIG2, and JPX image fixtures still require fallback under the
`image.filter` bucket. That is expected for this gate and remains separate from
the supported mobile-scan families.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `mobile-rotated-camera-scan.pdf` | rotation | camera scan image with `/Rotate 90` page metadata |
| `mobile-cropped-photo-scan.pdf` | crop | photo scan image with CropBox selection |
| `mobile-ocr-overlay-scan.pdf` | OCR layer | image page with invisible OCR text overlay |
| `mobile-mixed-compression-scan.pdf` | compression | Flate-compressed scan image plus DCT/JPEG image XObject |

`fixtures/mobile-scan-manifest.tsv` combines these with existing rotation,
cropped scan, invisible OCR, DCT, predictor, and unsupported image codec
baselines.

## Native Gate Evidence

Artifact: `target/mobile-scan-0129-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `rotation` | 2 | 2 | 0 | 0 |
| `crop` | 2 | 2 | 0 | 0 |
| `ocr-layer` | 2 | 2 | 0 | 0 |
| `compression` | 3 | 3 | 0 | 0 |
| **Total** | **9** | **9** | **0** | **0** |

The native regression tests also verify:

- rotated mobile scan metadata resolves to `320.0 x 240.0`;
- cropped mobile scan metadata resolves to `200.0 x 260.0`;
- invisible OCR text does not alter sampled scan pixels;
- image-dominant pages keep non-background output after thumbnail scaling.

## Unsupported Filter Backlog

Artifact: `target/mobile-scan-0129-unsupported-backlog.json`

| Family | Total | Native rendered | Fallback required | Fallback bucket | Errors |
| --- | ---: | ---: | ---: | --- | ---: |
| `unsupported-filter` | 3 | 0 | 3 | `image.filter` | 0 |

The backlog rows are `unsupported-ccitt-image.pdf`,
`unsupported-jbig2-image.pdf`, and `unsupported-jpx-image.pdf`.

## Benchmark Evidence

Artifact: `target/mobile-scan-0129-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `rotation` | 2 | 2 | 19.962 | 37.113 | 0 |
| `crop` | 2 | 2 | 46.012 | 51.042 | 0 |
| `ocr-layer` | 2 | 2 | 10.100 | 17.938 | 0 |
| `compression` | 3 | 3 | 10.306 | 28.705 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/mobile-scan-0129-visual-diff.json`

Thresholds: default strict visual review
`--max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `rotation` | 2 | 0 | 0 | 2 | 0 | 0 |
| `crop` | 2 | 1 | 0 | 1 | 0 | 0 |
| `ocr-layer` | 2 | 1 | 1 | 0 | 0 | 0 |
| `compression` | 3 | 2 | 0 | 1 | 0 | 0 |
| **Total** | **9** | **4** | **1** | **4** | **0** | **0** |

The blockers are visual-fidelity work, not native coverage failures:

- `mobile-rotated-camera-scan.pdf`: broad low-amplitude resampling drift.
- `rotated-office-export.pdf`: existing rotated text/vector raster drift.
- `mobile-cropped-photo-scan.pdf`: cropped image resampling drift.
- `mobile-mixed-compression-scan.pdf`: mixed Flate/JPEG image drift.

## Follow-Up Backlog

- Improve scan image resampling parity against PDFium.
- Add explicit mobile producer samples once redistribution and privacy review
  are cleared.
- Decide whether CCITT, JBIG2, and JPX should become native codecs or remain
  delegated/unsupported by policy.
- Add stress fixtures near image memory limits once large-image cache
  telemetry exists.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/mobile-scan-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p ferrugo-native mobile -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --fail-on-fallback --max-edge 160 --output target/mobile-scan-0129-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family unsupported-filter --max-edge 160 --output target/mobile-scan-0129-unsupported-backlog.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/mobile-scan-0129-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/mobile-scan-0129-visual-diff.json
```
