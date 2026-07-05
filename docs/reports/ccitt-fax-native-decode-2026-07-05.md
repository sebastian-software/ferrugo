# CCITT Fax Native Decode

Date: 2026-07-05
Issue: #103

## Summary

The native renderer now decodes `CCITTFaxDecode` and `/CCF` image streams for
the scan-family slice covered by the generated corpus:

- Group 3 1D (`K = 0`)
- mixed Group 3 1D/2D (`K > 0`)
- Group 4 2D (`K < 0`)
- `Columns`, `Rows`, `BlackIs1`, `EncodedByteAlign`, `EndOfLine`, and
  `EndOfBlock` decode parameters

Decoded output is budget checked before rasterization. Image masks remain packed
stencil data for the existing mask path, while 1-bit `/DeviceGray` CCITT images
expand to grayscale samples for the native image path.

## Fixture Coverage

| Fixture | Coverage |
| --- | --- |
| `ccitt-g3-1d-image-mask.pdf` | Group 3 1D image mask |
| `ccitt-g3-mixed-image-mask.pdf` | Mixed Group 3 1D/2D rows |
| `ccitt-g4-devicegray-blackis1.pdf` | Group 4 `/DeviceGray` image with `BlackIs1` |
| `ccitt-g3-1d-eol-aligned.pdf` | Group 3 1D with `EndOfLine`, `EncodedByteAlign`, and `EndOfBlock false` |
| `unsupported-ccitt-image.pdf` | Legacy path regenerated as a supported Group 3 1D image mask |

JPX and JBIG2 remain typed `image.filter` boundaries in
`fixtures/image-codec-deployment-manifest.tsv`.

## Validation

Focused validation run for this slice:

```sh
cargo test -p ferrugo-render ccitt -- --nocapture
cargo test -p ferrugo-render image_resources_should_report_unsupported_deferred_image_codecs -- --nocapture
cargo test -p ferrugo-native --no-default-features native_backend_should_render_generated_ccitt_fixture -- --nocapture
cargo test -p ferrugo-native --no-default-features image_codec_deployment -- --nocapture
cargo test -p ferrugo producer_regression_report_should_group_failures_by_producer_and_route -- --nocapture
bash scripts/check_codec_transparency_boundaries.sh
cargo run -p ferrugo --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/scanner-ocr-workflow-manifest.tsv --include-family skew --include-family large-image --include-family form-overlay --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family ccitt --fail-on-fallback --max-edge 160 --output target/issue103-scanner-ocr-supported.json
cargo run -p ferrugo --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/mobile-scan-manifest.tsv --include-family rotation --include-family crop --include-family ocr-layer --include-family compression --include-family ccitt --fail-on-fallback --max-edge 160 --output target/issue103-mobile-scan-supported.json
cargo run -p ferrugo --no-default-features -- render-native fixtures/generated/ccitt-g3-1d-image-mask.pdf --max-edge 120 --output target/issue103-ccitt-g3-1d.png
cargo run -p ferrugo --no-default-features -- render-native fixtures/generated/ccitt-g3-mixed-image-mask.pdf --max-edge 120 --output target/issue103-ccitt-g3-mixed.png
cargo run -p ferrugo --no-default-features -- render-native fixtures/generated/ccitt-g4-devicegray-blackis1.pdf --max-edge 120 --output target/issue103-ccitt-g4-blackis1.png
cargo run -p ferrugo --no-default-features -- render-native fixtures/generated/ccitt-g3-1d-eol-aligned.pdf --max-edge 120 --output target/issue103-ccitt-g3-eol-aligned.png
cargo test -p ferrugo-render
cargo test -p ferrugo-native
cargo test -p ferrugo
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```
