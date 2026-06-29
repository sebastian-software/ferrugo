# Vertical CJK Coverage 2026-06-24

## Scope

This report records milestone 0069 coverage for visible CJK text and vertical
Type0/CID text positioning in the native renderer.

## Commands

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/ferrugo-thumbnails/fallback-summary-0069.json

cargo run -p ferrugo-cli -- render-native fixtures/generated/vertical-cjk-text.pdf \
  --output target/ferrugo-thumbnails/vertical-cjk-text-native.png \
  --max-edge 180

FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/vertical-cjk-text.pdf \
  --output target/ferrugo-thumbnails/vertical-cjk-text-pdfium.png \
  --max-edge 180
```

## Results

Corpus fallback summary with the manifest:

| Scope | Total | Native rendered | Fallback required | Errors | Native pass rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| all generated fixtures | 44 | 42 | 1 | 1 encrypted | 0.955 |
| office-export family | 8 | 8 | 0 | 0 | 1.000 |

The new `vertical-cjk-text.pdf` fixture renders natively without fallback.
Native output dimensions are `180x120`, matching PDFium output dimensions. The
native output contains visible fallback glyph marks for the ToUnicode-mapped
Japanese text. PDFium produced a blank image for this synthetic fixture because
the fixture intentionally has no embedded CID font program; the comparison is
therefore useful for dimensions, not glyph visual parity.

Local RGBA comparison against that PDFium output reported mean absolute error
`3.046`, p95 channel delta `0`, and max channel delta `255`.

## Known Gaps

- Vertical positioning is structural: `Identity-V` text advances along the
  negative text Y axis, but glyph orientation and vertical glyph substitutions
  are not implemented.
- The fallback rasterizer draws visible placeholder glyphs for non-ASCII CJK
  characters; real CJK glyph outlines remain future work.
- Predefined CMaps, `/W2` vertical metrics, and embedded CID glyph ID mapping are
  not yet decoded.
