# Mixed Text Image Coverage 2026-06-24

## Scope

This report records milestone 0067 coverage for pages that combine selectable
text, image XObjects, and vector marks in one content stream.

## Commands

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/ferrugo-thumbnails/fallback-summary-0067.json

cargo run -p ferrugo-cli -- render-native fixtures/generated/mixed-text-image.pdf \
  --output target/ferrugo-thumbnails/mixed-text-image-native.png \
  --max-edge 220

FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/mixed-text-image.pdf \
  --output target/ferrugo-thumbnails/mixed-text-image-pdfium.png \
  --max-edge 220
```

## Results

Corpus fallback summary with the manifest:

| Scope | Total | Native rendered | Fallback required | Errors | Native pass rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| all generated fixtures | 42 | 40 | 1 | 1 encrypted | 0.952 |
| mixed-layout family | 9 | 8 | 0 | 1 encrypted | 0.889 |

The new `mixed-text-image.pdf` fixture renders natively without fallback. Native
and PDFium both produced `220x160` output. A local RGBA comparison reported mean
absolute error `11.194`, p95 channel delta `78`, and max channel delta `255`.
The structural z-order checks matched exactly:

| Pixel | Native | PDFium | Meaning |
| --- | --- | --- | --- |
| `(160, 64)` | `[180, 210, 245, 255]` | `[180, 210, 245, 255]` | image sample remains visible |
| `(160, 96)` | `[230, 51, 26, 255]` | `[230, 51, 26, 255]` | later vector marker paints over image |

## Known Gaps

- Text is still rendered through the fallback text rasterizer, so glyph shape and
  antialiasing remain visibly different from PDFium.
- Image and text clipping by path clips is not yet supported; current clip state
  applies to path-like items.
- Mixed Form XObject boundaries are still coarser than page-level path, image,
  and text ordering. Form-heavy mixed pages need a later boundary-aware display
  list pass.
