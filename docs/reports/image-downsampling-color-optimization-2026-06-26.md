# Image Downsampling And Color Conversion Optimization

Date: 2026-06-26
Milestone: 0137

## Summary

The image-heavy native rendering path now avoids two avoidable hot-path costs
without adding a full decoded RGBA image cache:

- PNG predictor rows are applied in-place on the decoded Flate sample buffer,
  then truncated to the exact decoded sample length.
- Scaled image painting uses a per-draw single-sample cache so repeated target
  pixels that map to the same source pixel reuse the last converted RGBA value.

Both changes keep memory bounded by existing image budgets. The renderer still
decodes the supported compressed stream into source samples before painting.
No compressed decode-window path was added in this slice because the current
supported Flate and DCT paths require whole-stream decode before PDF image
sample addressing is available. The native rasterizer already samples only the
target thumbnail pixels and does not materialize a full RGBA intermediate image.

## Corpus

The 0137 gate used `fixtures/mobile-scan-manifest.tsv` for scan, camera-photo,
OCR-over-image, mixed Flate/JPEG, DCT, and PNG predictor image coverage:

| Family | Rows | Coverage |
| --- | ---: | --- |
| `rotation` | 2 | Rotated camera scan plus office-export rotation baseline. |
| `crop` | 2 | Photo scan CropBox and cropped scan page. |
| `ocr-layer` | 2 | Image pages with invisible OCR text overlays. |
| `compression` | 3 | Mixed Flate/JPEG, DCT, and Flate PNG predictor images. |

Unsupported CCITT, JBIG2, and JPX rows remain excluded from the optimization
gate because 0137 does not change the specialized codec policy.

## Implementation

### In-Place PNG Predictor

`decode_image_samples` now transfers ownership of the decoded Flate buffer into
the PNG predictor path. Predictor reversal mutates row bytes in place and writes
decoded row bytes toward the front of the same allocation. This removes the
previous second full-size decoded-sample vector for predictor images.

### Repeated Sample Cache

`draw_image` now computes source sample coordinates once per target pixel and
uses `ImageSampleCache` for the currently painted image. The cache stores only
the last `(x, y, Rgba)` sample. This is enough to remove repeated CMYK, Indexed,
Gray, stencil-mask, and soft-mask conversion work during common thumbnail
upscaling runs, while keeping memory constant.

## Benchmark Evidence

Baseline before the 0137 changes:

| Family | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: |
| `compression` | 10.757 | 29.265 | 186240 |
| `crop` | 45.657 | 50.074 | 136320 |
| `ocr-layer` | 10.092 | 17.833 | 149120 |
| `rotation` | 19.831 | 36.643 | 140800 |

After in-place predictor and repeated-sample cache:

| Family | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: |
| `compression` | 10.821 | 30.494 | 186240 |
| `crop` | 46.766 | 51.248 | 136320 |
| `ocr-layer` | 10.483 | 18.496 | 149120 |
| `rotation` | 20.219 | 37.219 | 140800 |

The full supported image gate rendered all nine supported rows natively with
zero fallbacks, zero errors, and zero benchmark budget failures. The predictor
fixture specifically moved from 1.407 ms in the baseline run to 1.175 ms after
the in-place predictor change.

Runtime variance on the broader mobile scan families is larger than the
micro-optimization gain at `max_edge = 160`; the value of the slice is primarily
lower peak allocation in predictor decode and lower repeated color conversion
work when thumbnails upscale small image samples.

## Visual Comparison

The PDFium visual comparison completed without native or PDFium render errors.
Exact matches remain exact for DCT, predictor, cropped scan, and OCR overlay
fixtures. The comparison still reports the known PDFium resampling parity
blockers from the mobile scan corpus:

| Fixture | Status | MAE | P95 channel delta | Changed ratio |
| --- | --- | ---: | ---: | ---: |
| `cropped-scan-page.pdf` | exact | 0.000 | 0 | 0.000000 |
| `dct-image.pdf` | exact | 0.000 | 0 | 0.000000 |
| `mobile-cropped-photo-scan.pdf` | blocker | 1.471 | 17 | 0.267378 |
| `mobile-mixed-compression-scan.pdf` | blocker | 4.273 | 15 | 0.933615 |
| `mobile-ocr-overlay-scan.pdf` | exact | 0.000 | 0 | 0.000000 |
| `mobile-rotated-camera-scan.pdf` | blocker | 3.598 | 14 | 0.870833 |
| `ocr-invisible-text-layer.pdf` | accepted drift | 0.684 | 0 | 0.027371 |
| `predictor-image.pdf` | exact | 0.000 | 0 | 0.000000 |
| `rotated-office-export.pdf` | blocker | 16.074 | 171 | 0.193000 |

These blockers are not introduced by the 0137 memory and conversion changes;
they remain a separate image resampling fidelity backlog item.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p ferrugo-render image_ -- --nocapture
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --no-default-features
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --iterations 3 --max-edge 160 --max-ms 1000 --max-output-bytes 1048576 --output target/image-0137-final-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/image-0137-visual-diff.json
```

