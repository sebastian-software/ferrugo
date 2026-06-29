# 0088: Image Mask Stencil And Bitmap Edge Cases

Status: completed
Phase: 15
Size: medium
Depends on: 0087

## Goal

Handle common image mask and stencil patterns used in logos, signatures, icons,
and scanned overlays.

## Scope

- Implement ImageMask handling with current fill color.
- Cover decode inversion, one-bit masks, and stencil placement transforms.
- Add fixtures for signatures, monochrome icons, and masked logos.
- Preserve image memory budgets for large bitmap inputs.

## Non-Goals

- Add new compressed image codecs in this milestone.
- Support arbitrary color-managed proofing.
- Store expanded masks longer than needed for rendering.

## Deliverables

- ImageMask rendering support.
- Mask-focused fixture set.
- Memory notes for one-bit and expanded mask paths.

## Acceptance Criteria

- Stencil masks render with correct color and decode direction.
- Mask expansion is bounded and page-local unless explicitly cached.
- Visual comparisons match PDFium for typical masked bitmap documents.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run image-mask visual comparisons.
- Run large-mask memory checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Implemented `/ImageMask true` Image XObjects as stencil masks painted with
  the current fill color.
- Matched PDFium stencil polarity for `/Decode [0 1]` and `/Decode [1 0]`.
- Kept decoded mask samples row-packed and sampled bits directly during
  rasterization.
- Added signature, monochrome icon, and compressed logo ImageMask fixtures to
  the generated corpus and manifest.
- Added focused renderer tests for default polarity, inverted polarity,
  unsupported decode arrays, fill-color painting, and mask byte budgets.
- Added native fixture coverage for all three generated ImageMask documents.
- Wrote `docs/reports/image-mask-stencil-coverage-2026-06-25.md`.

Validation completed:

- `cargo fmt --check`
- `cargo test -p ferrugo-render image_mask`
- `cargo test -p ferrugo-native image_mask`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/image-mask-summary-0088.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/image-mask-visual-diff-0088.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/image-mask-benchmark-0088.json`

The ImageMask fixtures reported two exact visual matches and one accepted drift
against PDFium. The full corpus reported 61 fixtures total, 59 native renders,
1 known optional-content fallback, and 1 known encrypted error.
