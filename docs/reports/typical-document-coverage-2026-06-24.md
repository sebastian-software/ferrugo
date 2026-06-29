# Typical Document Coverage Gate

Date: 2026-06-24.
Milestone: 0040.

## Corpus Manifest

Committed seed fixtures stay under `fixtures/generated/` and cover deterministic
renderer behaviors. Real-world documents must stay local under
`fixtures/local-corpus/` and be described by copying
`fixtures/local-corpus.example.toml` to
`fixtures/local-corpus/metadata.toml`.

Local-only categories to track:

| Category | Example source | Expected policy |
| --- | --- | --- |
| Office export | locally generated `.docx`/`.pptx` export | render or typed unsupported |
| Browser PDF | print-to-PDF from Chromium/Safari | render or typed unsupported |
| Invoice | local synthetic or licensed-safe sample | render or typed unsupported |
| Scanned page | local generated scan-like PDF | render if image filters are supported |
| Image-heavy | camera/photo-heavy local PDF | render if image filters are supported |
| Vector-heavy | charts, diagrams, maps | render within path tolerance |
| Encrypted | local password-protected sample | encrypted |
| Malformed | reduced malformed sample | malformed or typed unsupported |

## Seed Results

Command shape:

```sh
cargo run -p ferrugo-cli -- render-native <fixture> --max-edge 256 --output target/ferrugo-thumbnails/<name>-native.png
```

PDFium oracle renders use:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli -- render <fixture> --max-edge <edge> --output <output.png>
```

| Fixture | Category | Native outcome | Evidence |
| --- | --- | --- | --- |
| `page-size-letter.pdf` | page geometry + rectangle fill | recognizable | native render succeeded at `max-edge 256` |
| `vector-paths.pdf` | vector-heavy | recognizable | 0037 pixel comparison: `220x180`, MAE `0.171`, p95 `0` |
| `text-page.pdf` | simple text | recognizable fallback | 0039 pixel comparison: `300x160`, MAE `12.082`, p95 `92`, visible fallback glyphs |
| `image-xobject.pdf` | Image XObject | parity for seed | 0038 pixel comparison: `120x120`, MAE `0.000`, p95 `0` |
| `form-xobject.pdf` | Form XObject invocation | unsupported in native render path | `render-native` returns `render error [unsupported]` |
| `inline-image.pdf` | inline image stream | failing gap | native render succeeds but outputs blank white image: `nonwhite=0` |

## Ranked Gaps

1. Inline image stream execution.
   Product impact: high for browser and report PDFs. Risk: medium; requires
   content tokenizer/interpreter support for `BI`/`ID`/`EI` image data.
   Follow-up: add an inline-image execution milestone before broader image
   filter work.

2. Form XObject invocation in the combined native render path.
   Product impact: high for generated office/browser PDFs. Risk: medium; Form
   display-list execution exists, but native rendering must merge nested
   path/text/image display items in content order.
   Follow-up: 0041 should classify this as the first integration gap.

3. Real font rendering.
   Product impact: high for text-heavy office/browser PDFs. Risk: high; the
   current ASCII fallback is visible but not typographically faithful.
   Follow-up: keep 0042-0045 as the font pipeline.

4. Additional image filters and color spaces.
   Product impact: high for scans and image-heavy PDFs. Risk: medium to high,
   depending on filter safety and memory expansion. Follow-up: 0046-0048.

5. Advanced stroke, clipping, transparency, and patterns.
   Product impact: medium for charts/design PDFs. Risk: medium to high.
   Follow-up: 0048-0051.

## Decision

The native renderer now covers the committed generated path, text fallback, and
Image XObject seeds well enough to move into gap triage. It is not ready to
replace PDFium for typical documents: inline images, combined Form XObject
rendering, real fonts, richer filters, and transparency remain visible product
gaps.
