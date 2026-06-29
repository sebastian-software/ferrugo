# 0029: Rust Backend Differential Harness

Status: done
Phase: 1
Size: medium
Depends on: 0028

## Goal

Create the test harness that compares Rust-native document behavior against the
PDFium oracle.

## Scope

- Add a Rust-native backend placeholder behind the thumbnail facade.
- Add a comparison command or test helper for fixture metadata.
- Compare page count, page size, error class, and later pixel output.
- Store comparison results in the existing baseline format.

## Non-Goals

- Render pixels with the Rust backend.
- Require PDFium in normal unit tests.
- Support the full real-world corpus in CI.

## Deliverables

- Rust-native backend adapter skeleton.
- Differential comparison harness.
- Baseline examples for metadata-only comparisons.

## Acceptance Criteria

- Generated fixtures can be compared against PDFium for page metadata.
- Mismatches produce actionable diagnostics.
- The harness can run without committing large rendered artifacts.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run one live comparison against the local PDFium dylib.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `feat: compare native metadata with pdfium` change.

- Added backend-neutral `DocumentMetadataBackend`, `DocumentMetadata`,
  `PageMetadata`, and `PageSize` types to `ferrugo-thumbnail`.
- Implemented metadata inspection in `ferrugo-pdfium` using `FPDF_GetPageCount`
  plus PDFium page width and height APIs.
- Implemented metadata inspection in `ferrugo-native` through the Rust object
  loader and `page_tree()` API while leaving pixel rendering unsupported.
- Added `ferrugo-cli compare-metadata` to compare PDFium oracle metadata against
  Rust-native metadata and emit compact JSON diagnostics.
- Added `baselines/examples/text-page-metadata-comparison.json` as the first
  metadata-only comparison baseline.
- Validation:
  - `cargo test -p ferrugo-thumbnail -p ferrugo-native -p ferrugo-pdfium -p ferrugo-cli`
  - Live PDFium comparison:
    `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- compare-metadata fixtures/generated/text-page.pdf --output target/ferrugo-thumbnails/text-page-metadata-comparison.json`
