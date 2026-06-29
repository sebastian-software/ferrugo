# 0089: JPX JBIG2 And Specialized Image Codec Policy

Status: completed
Phase: 15
Size: medium
Depends on: 0088

## Goal

Decide and implement the practical policy for specialized image codecs that
block native rendering of scanned and office-exported PDFs.

## Scope

- Inventory JPX, JBIG2, CCITT, and uncommon image filter usage in the corpus.
- Choose pure Rust, optional dependency, fallback, or unsupported handling per
  codec.
- Add deterministic errors for unsupported codec paths.
- Implement the highest-impact codec slice if the policy selects one.

## Non-Goals

- Ship unsafe decoder bindings without a reviewable safety boundary.
- Implement every rare image filter before measuring corpus impact.
- Silently fall back to PDFium in native-default mode.

## Deliverables

- Specialized codec decision record.
- Codec support or explicit unsupported-error implementation.
- Corpus report showing remaining image-codec blockers.

## Acceptance Criteria

- Each specialized image codec has a documented native strategy.
- Supported codecs respect memory and decompression budgets.
- Unsupported codecs produce actionable errors and support matrix entries.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run image-codec corpus classification.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added image filter alias handling for `/Fl` and `/DCT`.
- Kept `CCITTFaxDecode`/`CCF`, `JPXDecode`, and `JBIG2Decode` as explicit
  deferred codecs that map to `UnsupportedImageFilter`.
- Added generated unsupported-codec fixtures for CCITT, JBIG2, and JPX.
- Added native backend tests proving those fixtures map to `unsupported` with
  feature bucket `image.filter`.
- Recorded the policy in
  `docs/decisions/0006-specialized-image-codec-policy.md`.
- Wrote `docs/reports/specialized-image-codec-policy-2026-06-25.md`.

Validation completed:

- `cargo fmt --check`
- `cargo test -p ferrugo-render image_resources_should_decode_flate_alias_xobject`
- `cargo test -p ferrugo-render image_resources_should_route_dct_alias_to_jpeg_decoder`
- `cargo test -p ferrugo-render image_resources_should_report_unsupported_deferred_image_codecs`
- `cargo test -p ferrugo-native unsupported_`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/codec-policy-summary-0089.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/codec-policy-benchmark-0089.json`

The updated generated corpus reported 64 fixtures total: 59 native renders, 4
fallbacks, and 1 encrypted error. Three fallbacks are intentional `image.filter`
codec-policy fixtures; the remaining fallback is the known optional-content
policy case.
