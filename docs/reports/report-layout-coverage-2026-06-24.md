# Report Layout Coverage 2026-06-24

This report records milestone 0073 coverage for table-heavy reports and
multi-page document metadata in the Rust-native thumbnail renderer.

## Implemented Slice

- Added `fixtures/generated/multi-page-report.pdf`, a deterministic two-page
  report fixture with repeated header styling, a small logo marker, ruled table
  lines, and text cells.
- Added native-backend coverage that page 0 renders visibly at `260x160`.
- Added native-backend metadata coverage for page count, page dimensions, and
  document-order page indexes across two pages.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p ferrugo-native multi_page_report -- --nocapture
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/report-summary-0073.json
cargo run -p ferrugo-cli -- render-native fixtures/generated/multi-page-report.pdf --max-edge 260 --output target/ferrugo-thumbnails/multi-page-report-native.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/multi-page-report.pdf --max-edge 260 --output target/ferrugo-thumbnails/multi-page-report-pdfium.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- compare-metadata fixtures/generated/multi-page-report.pdf --output target/ferrugo-thumbnails/multi-page-report-metadata-0073.json
```

All commands completed successfully.

The generated corpus summary reported 48 fixtures total, 46 native renders, 1
native fallback requirement for optional content policy, and 1 encrypted input
classification. The `office-export` family, including the new multi-page report
fixture, rendered 10 of 10 fixtures natively.

`compare-metadata` matched PDFium exactly for the new report fixture:

```text
page_count: 2
page 0: 260.000 x 160.000
page 1: 240.000 x 180.000
```

Native and PDFium rendered page 0 at `260x160`. Local RGBA comparison reported
mean absolute channel delta `11.522`, p95 channel delta `91`, and max channel
delta `255`. The high deltas are expected in this slice because native text is
still drawn through the fallback bitmap glyph policy while PDFium uses its font
renderer; table ruling lines, header blocks, and text placeholders remain
visible.

## Remaining Limits

- Native rendering still thumbnails page 0 only; multi-page rendering beyond
  metadata comparison needs an explicit facade/API extension.
- Small text legibility is visible but not typographically faithful until the
  glyph/font pipeline replaces fallback bitmap drawing for common fonts.
- Real invoices and statements still need local-corpus sampling for logos,
  repeated headers, and mixed image/text/table pages.
