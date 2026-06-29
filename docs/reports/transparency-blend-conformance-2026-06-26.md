# Transparency And Blend Conformance Corpus

Date: 2026-06-26
Milestone: 0138

## Summary

Milestone 0138 adds a focused transparency conformance manifest and expands the
native regression corpus for common transparency, blend, and soft-mask
boundaries.

Added fixtures:

- `transparency-isolated-alpha-group.pdf`
- `blend-mode-array-fallback.pdf`
- `unsupported-blend-mode.pdf`
- `extgstate-luminosity-soft-mask.pdf`

The new manifest is `fixtures/transparency-conformance-manifest.tsv`. The same
fixtures are registered in `fixtures/corpus-manifest.tsv`.

## Support Matrix

| Feature | Fixture | Native behavior |
| --- | --- | --- |
| ExtGState fill/stroke alpha | `transparency-alpha.pdf` | native |
| Isolated transparency group | `transparency-group.pdf` | native |
| Isolated alpha group overlap | `transparency-isolated-alpha-group.pdf` | native |
| Knockout group metadata | `transparency-knockout-group.pdf` | native, PDFium-guided overlap behavior |
| Multiply and Screen blend modes | `blend-modes.pdf` | native |
| Blend-mode array with unsupported first item | `blend-mode-array-fallback.pdf` | native, falls through to supported mode |
| Image soft mask | `soft-mask-image.pdf` | native |
| ExtGState luminosity soft mask | `extgstate-luminosity-soft-mask.pdf` | typed `graphics.transparency` fallback |
| Overlay blend mode | `unsupported-blend-mode.pdf` | typed `graphics.transparency` fallback |

`/SMask /None` is accepted in ExtGState dictionaries. Other ExtGState soft masks
are now explicitly classified instead of being ignored.

## Native Gates

Supported-family fallback gate:

| Family | Total | Native | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: |
| `alpha` | 1 | 1 | 0 | 0 |
| `blend` | 2 | 2 | 0 | 0 |
| `group` | 3 | 3 | 0 | 0 |
| `image-soft-mask` | 1 | 1 | 0 | 0 |

Unsupported boundary gate:

| Family | Total | Native | Fallbacks | Bucket |
| --- | ---: | ---: | ---: | --- |
| `unsupported-blend` | 1 | 0 | 1 | `graphics.transparency` |
| `unsupported-soft-mask` | 1 | 0 | 1 | `graphics.transparency` |

## Benchmark

Artifact: `target/transparency-0138-benchmark.json`

| Family | Total | Native | Fallbacks | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `alpha` | 1 | 1 | 0 | 0 | 32.569 | 32.569 | 57600 |
| `blend` | 2 | 2 | 0 | 0 | 28.253 | 28.820 | 115200 |
| `group` | 3 | 3 | 0 | 0 | 19.457 | 28.826 | 172800 |
| `image-soft-mask` | 1 | 1 | 0 | 0 | 1.179 | 1.179 | 57600 |
| `unsupported-blend` | 1 | 0 | 1 | 1 | 0.000 | 0.000 | 0 |
| `unsupported-soft-mask` | 1 | 0 | 1 | 1 | 0.000 | 0.000 | 0 |

The two budget failures are expected fallback-boundary rows. Supported
transparency families have zero benchmark budget failures.

## Visual Diff

Artifact: `target/transparency-0138-visual-diff.json`

| Fixture | Status | MAE | P95 delta | Changed ratio |
| --- | --- | ---: | ---: | ---: |
| `blend-mode-array-fallback.pdf` | exact | 0.000 | 0 | 0.000000 |
| `blend-modes.pdf` | exact | 0.000 | 0 | 0.000000 |
| `soft-mask-image.pdf` | exact | 0.000 | 0 | 0.000000 |
| `transparency-group.pdf` | exact | 0.000 | 0 | 0.000000 |
| `transparency-isolated-alpha-group.pdf` | accepted drift | 0.167 | 1 | 0.437500 |
| `transparency-knockout-group.pdf` | accepted drift | 0.167 | 1 | 0.437500 |
| `transparency-alpha.pdf` | blocker | 0.269 | 1 | 0.200000 |
| `extgstate-luminosity-soft-mask.pdf` | native error | n/a | n/a | n/a |
| `unsupported-blend-mode.pdf` | native error | n/a | n/a | n/a |

The remaining visual blocker is the pre-existing `transparency-alpha.pdf`
stroke-edge max-delta issue documented in earlier transparency reports. The new
supported group and blend fixtures are exact or accepted drift. The two native
errors are intentional typed unsupported boundaries.

## Limits

- This is not full print-proof transparency conformance.
- ExtGState luminosity soft masks are now explicit unsupported boundaries.
- Overlay and other advanced blend modes remain unsupported until corpus
  evidence justifies implementation.
- True nested Form XObject transparency groups remain out of this fixture slice;
  intermediate-surface allocation remains bounded by
  `PathRasterOptions::max_transparency_group_pixels` and covered by renderer
  budget tests.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p ferrugo-render ext_graphics_state_resources -- --nocapture
cargo test -p ferrugo-native transparency -- --nocapture
cargo test -p ferrugo-native blend_mode -- --nocapture
cargo test -p ferrugo-native isolated_alpha_group -- --nocapture
cargo test -p ferrugo-native extgstate_luminosity -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/transparency-conformance-manifest.tsv --include-family alpha --include-family group --include-family blend --include-family image-soft-mask --fail-on-fallback --max-edge 160 --output target/transparency-0138-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/transparency-conformance-manifest.tsv --include-family unsupported-soft-mask --include-family unsupported-blend --max-edge 160 --output target/transparency-0138-unsupported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/transparency-conformance-manifest.tsv --include-family alpha --include-family group --include-family blend --include-family image-soft-mask --include-family unsupported-soft-mask --include-family unsupported-blend --iterations 2 --max-edge 160 --max-ms 1000 --max-output-bytes 1048576 --output target/transparency-0138-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/transparency-conformance-manifest.tsv --include-family alpha --include-family group --include-family blend --include-family image-soft-mask --include-family unsupported-soft-mask --include-family unsupported-blend --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/transparency-0138-visual-diff.json
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --no-default-features
```
