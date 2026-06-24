# Native Renderer Support Matrix

Date: 2026-06-24.
Milestone: 0041.

## Support Levels

| Level | Meaning | PDFium role |
| --- | --- | --- |
| `rendered` | Native output is recognizable and within the fixture tolerance. | Oracle and regression baseline. |
| `degraded` | Native output is useful for thumbnails but known to differ visibly. | Fallback for quality-sensitive callers. |
| `unsupported` | The PDF is valid, but a required feature is not implemented yet. | Required for production rendering. |
| `malformed` | The input cannot be parsed as valid PDF within the recovery budget. | Oracle for error classification only. |
| `encrypted` | The input needs a password or unsupported security handling. | Required until native encryption policy lands. |

## Seed Fixture Matrix

| Fixture | Current level | Feature owner | Next owner milestone | PDFium required? |
| --- | --- | --- | --- | --- |
| `page-size-letter.pdf` | `rendered` | page geometry, solid fills | done in 0036-0037 | no |
| `vector-paths.pdf` | `rendered` | path rasterization | done in 0037 | no for simple paths |
| `text-page.pdf` | `degraded` | fallback ASCII text | 0042-0045 font pipeline | yes for faithful text |
| `image-xobject.pdf` | `rendered` | DeviceRGB Image XObject | done in 0038 | no for unfiltered RGB/gray images |
| `form-xobject.pdf` | `rendered` | path-only Form XObject composition | done in 0041b, then 0059 parity | no for path-only forms |
| `inline-image.pdf` | `rendered` | unfiltered inline image stream execution | done in 0041a | no for unfiltered RGB/gray inline images |

## Local Corpus Category Matrix

| Category | Expected native level now | Dominant missing features | Owner milestone |
| --- | --- | --- | --- |
| Office export | `unsupported` or `degraded` | fonts, CMaps, Form XObjects, transparency | 0042-0045, 0048 |
| Browser PDF | `unsupported` or `degraded` | inline images, fonts, transparency, clipping | pull forward inline images, 0042-0045, 0048, 0051 |
| Invoice | `degraded` | fonts, barcodes/images, annotation appearances | 0042-0047, 0052 |
| Scanned page | `unsupported` | CCITT/JPX/JBIG2 filters, soft masks, memory limits | 0047, 0048, 0058 |
| Image-heavy | `unsupported` | codecs, color spaces, soft masks | 0046-0048 |
| Vector-heavy | `degraded` | clipping fidelity, stroke joins/caps, patterns | 0050-0051 |
| Encrypted | `encrypted` | security handler and permission policy | 0056 |
| Malformed | `malformed` or `unsupported` | repair budget, hybrid refs, incremental updates | 0055, 0057 |

## Ranked Backlog

| Rank | Gap | Product value | Implementation risk | Memory risk | Owner |
| --- | --- | --- | --- | --- | --- |
| 1 | Font program loading and fallback policy | high | high | medium | 0042 |
| 2 | CMap and ToUnicode mapping | high | high | low to medium | 0043 |
| 3 | Glyph outline extraction | high | high | medium | 0044 |
| 4 | Complex text positioning baseline | high | medium | low | 0045 |
| 5 | Color spaces and decode arrays | high | medium | low | 0046 |
| 6 | Image filter coverage | high | medium to high | high | 0047 |
| 7 | Text and image execution inside Form XObjects | medium to high | medium | medium | 0059 parity follow-up |
| 8 | Soft masks and transparency groups | medium to high | high | high | 0048 |
| 9 | Advanced stroke and clipping fidelity | medium | medium | low | 0051 |

## Error Taxonomy Decision

No new public thumbnail error class is needed. Valid PDFs blocked by native
renderer feature gaps still map to `unsupported`; corpus and diagnostics should
record a stable internal feature bucket such as `renderer.inline-image-stream`,
`renderer.form-xobject-composition`, `text.font-program`, `image.filter`, or
`graphics.transparency`.

## Planning Note

The existing milestone chain covers the font, image, graphics, document, cache,
facade, and retirement gates. The 0040 evidence adds two higher-priority
integration gaps that should be pulled forward before the font pipeline or
image-filter work: inline image streams and combined Form XObject rendering.
The unfiltered inline-image slice landed as 0041a; filtered inline images remain
part of 0047 image filter coverage.
Path-only native Form XObject composition landed as 0041b. Text and image
execution inside forms remains a later parity gap.
