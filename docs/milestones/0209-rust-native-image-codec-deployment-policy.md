# 0209: Rust-Native Image Codec Deployment Policy

Status: done
Phase: 39
Size: medium
Depends on: 0208

## Goal

Define and validate the Rust-native image codec deployment policy for common
PDF image content across desktop, server, WASM, and low-memory profiles.

## Scope

- Audit JPEG, JPEG 2000, JBIG2, CCITT, Flate, image masks, decode arrays, and
  color-space interactions in the supported corpus.
- Separate built-in Rust decoders, optional native dependencies, and unsupported
  codec boundaries.
- Measure codec memory behavior on large scanned and image-heavy documents.
- Add deployment guidance for profiles that cannot ship specialized codecs.

## Non-Goals

- Accept unsafe decoders without isolation or fuzz evidence.
- Implement every rare image codec before it appears in typical documents.
- Reintroduce PDFium solely for specialized image decoding.

## Deliverables

- Rust-native image codec deployment policy.
- Codec coverage and unsupported-boundary matrix.
- Memory and security report for image-heavy documents.

## Acceptance Criteria

- Common image-heavy documents have a clear native codec path.
- Optional codec dependencies are explicit and profile-specific.
- Unsupported image features produce typed, user-visible diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run image codec corpus comparisons.
- Run scanned-document memory benchmark.
- Run codec fuzz smoke suite.
- Run package profile checks for desktop, server, and WASM.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `fixtures/image-codec-deployment-manifest.tsv` to split supported
  built-in Rust image paths from deferred specialized codecs.
- Added
  `native_backend_should_enforce_image_codec_deployment_policy` to verify raw,
  inline, Flate predictor, mixed Flate/DCT, DCT/JPEG, image-mask, soft-mask,
  and large-scan paths render natively while CCITT, JBIG2, and JPX stay typed
  `image.filter` unsupported boundaries.
- Extended `docs/decisions/0006-specialized-image-codec-policy.md` with a
  desktop/server/WASM deployment matrix.
- Documented the manifest in `docs/corpus-taxonomy.md`.
- Produced `docs/reports/rust-native-image-codec-deployment-2026-06-29.md`.

Validation run:

- `cargo fmt --check`
- `cargo test -p ferrugo-native image_codec_deployment -- --nocapture`
- `cargo test -p ferrugo-render image_resources_should_report_unsupported_deferred_image_codecs -- --nocapture`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --fail-on-fallback --max-edge 180 --output target/image-codec-0209-supported.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family unsupported-specialized --max-edge 180 --output target/image-codec-0209-unsupported.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --max-edge 180 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/image-codec-0209-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/image-codec-deployment-manifest.tsv --include-family builtin-raster --include-family flate-predictor --include-family mixed-compression --include-family jpeg --include-family mask-alpha --include-family image-heavy --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/image-codec-0209-poppler.json`
- `bash scripts/check_fuzz_smoke.sh`
- `bash scripts/check_native_only_release.sh`
- `bash scripts/check_wasm_smoke.sh`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
