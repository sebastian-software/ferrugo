# Transparency Stack Coverage 2026-06-24

This report records milestone 0071 coverage for the next transparency slice in
the Rust-native thumbnail renderer.

## Implemented Slice

- `/ExtGState` now decodes nonstroking `/ca` and stroking `/CA` alpha constants.
- The display-list graphics state snapshots preserve fill and stroke alpha
  independently.
- Path fill, tiling-pattern fill, and stroke rasterization multiply alpha by
  supersampling coverage before compositing into the raster device.
- `transparency-alpha.pdf` covers a gray backdrop, half-alpha red fill, and
  half-alpha blue stroke through ExtGState resources.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p ferrugo-render form_transparency_group_should_enforce_intermediate_pixel_budget -- --nocapture
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/transparency-summary-0071.json
cargo run -p ferrugo-cli -- render-native fixtures/generated/transparency-alpha.pdf --max-edge 120 --output target/ferrugo-thumbnails/transparency-alpha-native.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/transparency-alpha.pdf --max-edge 120 --output target/ferrugo-thumbnails/transparency-alpha-pdfium.png
```

All commands completed successfully. A direct `render-pdfium` run without
`FERRUGO_PDFIUM_LIBRARY` failed first with the expected environment error, then
passed with the documented local PDFium dylib.

The generated corpus summary reported 46 fixtures total, 44 native renders, 1
native fallback requirement for optional content policy, and 1 encrypted input
classification. The `report` family, including the new alpha fixture, rendered
9 of 9 fixtures natively.

Native and PDFium rendered `transparency-alpha.pdf` at `120x120`. Local RGBA
comparison reported mean absolute channel delta `0.202`, p95 channel delta `1`,
and max channel delta `64`. The max delta occurs on anti-aliased stroke edges;
major fill and stroke layers are present in both outputs.

## Remaining Limits

- Text alpha is not yet applied to fallback glyph rasterization.
- Soft masks remain image-only; luminosity soft masks for arbitrary content are
  still outside this slice.
- Transparency groups are bounded and rasterized through intermediate buffers,
  but isolated/knockout semantics remain path-focused rather than full
  prepress-grade compositing.
- Advanced blend modes beyond `Normal`, `Multiply`, and `Screen` still produce
  typed unsupported errors.
