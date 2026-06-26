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
| Scanned page | `unsupported` | CCITT/JPX/JBIG2 filters, large-image memory limits | 0047, 0058 |
| Image-heavy | `unsupported` | codecs, color spaces, transparency groups | 0046-0048 |
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

## RC 0080 Update

The 0080 release-candidate gate supersedes the early 0041 category estimates
for committed generated fixtures.

| Category | 0080 generated-corpus status | Native blocker | PDFium fallback status |
| --- | --- | --- | --- |
| browser-print | 4/4 rendered | visual text fidelity still needs diff review | keep as oracle |
| form | 6/6 rendered | visual widget fidelity still needs diff review | keep as oracle |
| mixed-layout | 8/9 rendered, 1 encrypted expected error | encrypted policy, not render fallback | no fallback blocker |
| office-export | 10/10 rendered | text/table visual fidelity still needs diff review | keep for quality-sensitive comparison |
| presentation | 3/4 rendered | `optional-content-ocmd.pdf` needs `graphics.optional-content` fallback | required |
| report | 12/12 rendered | `vector-stress.pdf` exceeds smoke render-time budget | keep as benchmark oracle |
| scan | focused scanner/OCR supported manifest 10/10 rendered | main scan family still contains 3 deferred codec-policy rows | keep as oracle |

Release blockers from 0080:

1. Optional-content membership policy fallback in the presentation family.
2. Missing full-corpus visual diff workflow.
3. `vector-stress.pdf` render-time budget failure.
4. Text-heavy visual fidelity risk without automated comparison thresholds.

The native renderer can remain native-first for controlled categories, but
PDFium is still required as oracle and fallback until these blockers are closed.

## 0089 Codec Policy Update

Milestone 0089 makes specialized image codec handling explicit:

| Codec/filter | Native level | Feature bucket | Policy |
| --- | --- | --- | --- |
| `FlateDecode`, `Fl` | `rendered` | none | Existing safe stream decoder and predictor path. |
| `DCTDecode`, `DCT` | `rendered` | none | Existing safe Rust JPEG decoder path. |
| `CCITTFaxDecode`, `CCF` | `unsupported` | `image.filter` | Deferred until safe decoder selection and scan corpus evidence. |
| `JPXDecode` | `unsupported` | `image.filter` | Deferred until safe or isolated JPEG 2000 decoder selection. |
| `JBIG2Decode` | `unsupported` | `image.filter` | Deferred until sandboxed or strongly isolated decoder strategy. |

The generated corpus now includes `unsupported-ccitt-image.pdf`,
`unsupported-jbig2-image.pdf`, and `unsupported-jpx-image.pdf` as deterministic
codec-policy blockers. They are expected native fallbacks, not malformed-input
errors.

## 0143 Conformance Triage Update

Milestone 0143 supersedes the broad RC-era visual-risk language with measured
subsystem tags from `target/conformance-0143-visual-diff.json`.

Core supported runtime families remain native-only:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| browser-print | 11 | 11 | 0 | 0 |
| form | 18 | 18 | 0 | 0 |
| office-export | 47 | 47 | 0 | 0 |

Visual conformance still has 91 blockers and 8 native unsupported rows in the
0143 full-corpus baseline. The 0145 office-only refresh adds three
rendering-core blockers while keeping the office native-supported gate green.
The 0146 browser-print refresh adds three browser rendering-core blockers while
keeping the browser native-supported and repeat-render gates green.
The 0147 scanner/OCR refresh adds skew, large-image, and OCR form-overlay
reductions while keeping the supported scanner workflow gate green.
The 0148 government/form refresh adds permit, certificate, and tax-notice
reductions while keeping the supported form-family gate green.

| Subsystem tag | Blockers | Native errors | Primary owner |
| --- | ---: | ---: | --- |
| `rendering-core` | 34 | 1 | Dense tables, reports, dashboards, and XFA policy boundaries. |
| `text-fonts` | 24 | 0 | Font metrics, spacing, fallback glyphs, and subset width parity. |
| `annotations-forms` | 13 | 0 | Form widget and annotation appearance parity. |
| `page-geometry` | 9 | 0 | Rotation, crop, user-unit, and first-page transform parity. |
| `images-color` | 6 | 3 | Image resampling, CMYK/ICC drift, and unsupported codec policy. |
| `vector-graphics` | 3 | 1 | High-delta vector/shading blockers distinct from accepted gradient drift. |
| `transparency` | 1 | 2 | Alpha drift plus soft-mask/blend unsupported boundaries. |
| `document-structure` | 1 | 0 | Hybrid reference visual parity. |
| `optional-content` | 0 | 1 | OCMD membership and layer flattening policy. |
| `document-security` | 0 | 0 | Encrypted inputs remain policy boundaries, not visual blockers. |

Follow-up routing is tracked in
`docs/backlogs/native-renderer-conformance-backlog.md`. The stable report
contract is documented in `docs/policies/native-conformance-triage.md`.

## 0144 Operator Coverage Update

The 0144 operator audit adds measured content-stream operator usage for the
generated corpus. The scanner reports 154 scanned fixtures, 1 encrypted policy
error, and 5,565 total operators.

| Status | Count | Owner implication |
| --- | ---: | --- |
| `implemented` | 5,472 | Common path, text, image, and graphics-state operators are recognized by native rendering. |
| `partial` | 85 | Focus follow-up work on `gs`, `W` / `W*`, color-space operators, and `sh`. |
| `unsupported` | 0 | The current generated corpus does not exercise fully unsupported content-stream operators. |
| `ignored` | 8 | Marked-content operators are non-visual for thumbnail output. |

Operator follow-up priority:

1. `graphics.stroke-clip`: clipping parity for `W` / `W*`.
2. `graphics.transparency`: external graphics-state subset validation for `gs`.
3. `image.color-space`: `cs`, `CS`, `scn`, and `SCN` parity.
4. `graphics.pattern-shading`: shading and pattern coverage for `sh`.

Report: `docs/reports/renderer-operator-coverage-audit-2026-06-26.md`.
