# CID Font Coverage 2026-06-24

## Scope

This report records milestone 0068 coverage for Type0 composite fonts backed by
CID descendant font metadata.

## Commands

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/pdfrust-thumbnails/fallback-summary-0068.json

cargo run -p pdfrust-cli -- render-native fixtures/generated/cid-font-text.pdf \
  --output target/pdfrust-thumbnails/cid-font-text-native.png \
  --max-edge 180

PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli -- render-pdfium fixtures/generated/cid-font-text.pdf \
  --output target/pdfrust-thumbnails/cid-font-text-pdfium.png \
  --max-edge 180
```

## Results

Corpus fallback summary with the manifest:

| Scope | Total | Native rendered | Fallback required | Errors | Native pass rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| all generated fixtures | 43 | 41 | 1 | 1 encrypted | 0.953 |
| office-export family | 7 | 7 | 0 | 0 | 1.000 |

The new `cid-font-text.pdf` fixture renders natively without fallback. Native
and PDFium both produced `180x100` output. A local RGBA comparison reported mean
absolute error `3.869`, p95 channel delta `0`, and max channel delta `255`.

## Known Gaps

- `/DW` default widths are supported for Type0/CID fallback text advance, but
  `/W` per-CID width arrays are not yet decoded.
- `Identity-H` is supported for horizontal Type0 text with ToUnicode mapping.
  Vertical `Identity-V` behavior is deferred to milestone 0069.
- Glyph painting still uses the fallback text rasterizer. CID-aware glyph ID to
  embedded outline mapping remains future work.
