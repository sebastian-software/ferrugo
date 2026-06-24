# Shaped Text Policy Coverage 2026-06-24

## Scope

This report records milestone 0070 coverage for bidirectional and shaped text
policy in the native renderer.

## Commands

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/pdfrust-thumbnails/fallback-summary-0070.json

cargo run -p pdfrust-cli -- render-native fixtures/generated/shaped-rtl-text.pdf \
  --output target/pdfrust-thumbnails/shaped-rtl-text-native.png \
  --max-edge 180

PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli -- render-pdfium fixtures/generated/shaped-rtl-text.pdf \
  --output target/pdfrust-thumbnails/shaped-rtl-text-pdfium.png \
  --max-edge 180
```

## Results

Corpus fallback summary with the manifest:

| Scope | Total | Native rendered | Fallback required | Errors | Native pass rate |
| --- | ---: | ---: | ---: | ---: | ---: |
| all generated fixtures | 45 | 43 | 1 | 1 encrypted | 0.956 |
| office-export family | 9 | 9 | 0 | 0 | 1.000 |

The new `shaped-rtl-text.pdf` fixture renders natively without fallback. Native
and PDFium both produced `180x100` visible output. A local RGBA comparison
reported mean absolute error `7.952`, p95 channel delta `0`, and max channel
delta `255`. The remaining difference is expected while native uses fallback
glyph drawing instead of real Hebrew glyph outlines.

## Policy Summary

The accepted policy is recorded in
`docs/decisions/0004-bidirectional-and-shaped-text-policy.md`: native rendering
follows pre-shaped PDF glyph codes and text positions. It does not shape Unicode
source text before layout in this phase.

## Known Gaps

- Fallback glyph drawing is visible but not typographically faithful for complex
  scripts.
- Missing glyph outlines, unsupported encodings, and unsupported CMaps remain
  the typed fallback boundaries for shaped-text documents.
- Adding a shaping dependency requires separate corpus and benchmark evidence.
