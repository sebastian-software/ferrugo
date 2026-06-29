# Rust-Native Image Codec Deployment Policy

Date: 2026-06-29
Milestone: 0209

## Decision

The default Rust-native deployment path stays plugin-free and PDFium-free for
common image-heavy documents. Raw images, inline images, Flate+predictor image
streams, mixed Flate/DCT scans, DCT/JPEG, image masks, soft masks, and large
Flate scan pages are supported by the built-in native path.

CCITT Fax, JPEG 2000, and JBIG2 remain explicit deferred codecs. They must keep
returning typed `image.filter` diagnostics until a future slice selects safe
decoder implementations or an isolated out-of-process strategy with fuzz and
memory evidence.

## Corpus

0209 adds `fixtures/image-codec-deployment-manifest.tsv`.

| Family | Fixtures | Policy |
| --- | ---: | --- |
| `builtin-raster` | 2 | Raw Image XObject and inline image support. |
| `flate-predictor` | 1 | Flate image stream with PNG predictor support. |
| `mixed-compression` | 1 | Mobile scan using mixed Flate and DCT images. |
| `jpeg` | 1 | DCTDecode JPEG support through the safe Rust decoder path. |
| `mask-alpha` | 2 | ImageMask and soft-mask alpha support. |
| `image-heavy` | 1 | Large compressed scan memory-budget path. |
| `unsupported-specialized` | 3 | CCITT, JBIG2, and JPX typed `image.filter` boundaries. |

## Supported Runtime Gate

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --fail-on-fallback --max-edge 180 --output target/image-codec-0209-supported.json
```

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 0 |

## Deferred Codec Boundary

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family unsupported-specialized --max-edge 180 --output target/image-codec-0209-unsupported.json
```

| Total | Native rendered | Fallbacks | Fallback category | Errors |
| ---: | ---: | ---: | --- | ---: |
| 3 | 0 | 3 | `image.filter` | 0 |

## Benchmark

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/image-codec-0209-benchmark.json
```

| Total | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 0 | 0 |

## Visual Review

Poppler is used only as an independent review oracle.

```sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/image-codec-0209-poppler.json
```

| Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0 | 7 | 1 | 0 | 0 |

The blocker is `mobile-mixed-compression-scan.pdf`: native and reference both
produce non-white output for the same full page, with low p95 channel delta but
high changed-ratio. Track it as scan resampling/layout visual review, not as a
codec deployment blocker.

## Package And Security Gates

| Gate | Result |
| --- | --- |
| Fuzz smoke | Passed: primitive parse, xref load, stream decode, content tokenize, render setup. |
| Native-only release | Passed: native check/test, plugin-free distribution, PDFium quarantine, dry-run packages, all-features clippy. |
| WASM smoke | Passed: 728680-byte wasm artifact, compile 2.305 ms, instantiate 0.089 ms, smoke 5.886 ms. |

## Validation

Commands run:

```text
cargo fmt --check
cargo test -p ferrugo-native image_codec_deployment -- --nocapture
cargo test -p ferrugo-render image_resources_should_report_unsupported_deferred_image_codecs -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --fail-on-fallback --max-edge 180 --output target/image-codec-0209-supported.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family unsupported-specialized --max-edge 180 --output target/image-codec-0209-unsupported.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/image-codec-0209-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/image-codec-0209-poppler.json
bash scripts/check_fuzz_smoke.sh
bash scripts/check_native_only_release.sh
bash scripts/check_wasm_smoke.sh
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
