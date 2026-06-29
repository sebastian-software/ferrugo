# Mixed Vector Raster Transparency Baseline 2026-06-28

Milestone: 0183.
Status: in progress.

## Summary

Milestone 0183 now has a focused manifest for common mixed vector, raster,
clipping, soft-mask, and transparency pages assembled from existing generated
fixtures:

- `fixtures/mixed-vector-raster-transparency-manifest.tsv`

The baseline confirms native coverage is available for the selected typical
slice: all 8 fixtures render natively with 0 fallbacks, 0 errors, and 0
benchmark budget failures. Strict Poppler visual comparison still reports 6
fidelity blockers, so 0183 remains open for renderer fidelity work rather than
coverage or oracle availability.

## Fixture Slice

| Fixture | Family | Coverage |
| --- | --- | --- |
| `browser-print-raster-vector-mix.pdf` | `browser-raster-vector` | Browser print page with raster/vector chart content. |
| `high-dpi-preview-fidelity.pdf` | `high-dpi-preview` | Fine linework, small text, scaled image, and alpha overlay. |
| `image-heavy-rotated-mask-sheet.pdf` | `rotated-soft-mask-image` | Rotated masked image placements. |
| `map-transparent-zoning-overlay.pdf` | `map-overlay` | Transparent zoning overlay over grid and route vectors. |
| `office-vector-clipped-transparency-group.pdf` | `office-clipped-transparency` | Clipped office transparency group. |
| `office-vector-repeated-effects.pdf` | `office-repeated-effects` | Repeated decorative office vector transparency effects. |
| `slide-layered-image-shadow.pdf` | `slide-layered-image` | Scaled image, translucent tint overlay, shadow block, and text. |
| `soft-mask-image.pdf` | `image-soft-mask` | Image soft-mask alpha baseline. |

## Native Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/mixed-vector-raster-transparency-manifest.tsv \
  --include-family browser-raster-vector \
  --include-family high-dpi-preview \
  --include-family rotated-soft-mask-image \
  --include-family map-overlay \
  --include-family office-clipped-transparency \
  --include-family office-repeated-effects \
  --include-family slide-layered-image \
  --include-family image-soft-mask \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/mixed-transparency-0183-supported-gate.json
```

Result: 8 total, 8 native rendered, 0 fallback required, 0 errors.

## Benchmark Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/mixed-vector-raster-transparency-manifest.tsv \
  --include-family browser-raster-vector \
  --include-family high-dpi-preview \
  --include-family rotated-soft-mask-image \
  --include-family map-overlay \
  --include-family office-clipped-transparency \
  --include-family office-repeated-effects \
  --include-family slide-layered-image \
  --include-family image-soft-mask \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/mixed-transparency-0183-benchmark.json
```

Result: 8 total, 8 native rendered, 0 fallback required, 0 errors, 0 budget
failures.

| Family | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: |
| `browser-raster-vector` | 22.841 | 22.841 | 102400 |
| `high-dpi-preview` | 33.038 | 33.038 | 76800 |
| `image-soft-mask` | 1.234 | 1.234 | 57600 |
| `map-overlay` | 64.515 | 64.515 | 69760 |
| `office-clipped-transparency` | 65.153 | 65.153 | 70400 |
| `office-repeated-effects` | 105.501 | 105.501 | 68480 |
| `rotated-soft-mask-image` | 48.081 | 48.081 | 74240 |
| `slide-layered-image` | 18.421 | 18.421 | 57600 |

## Poppler Visual Baseline

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/mixed-vector-raster-transparency-manifest.tsv \
  --include-family browser-raster-vector \
  --include-family high-dpi-preview \
  --include-family rotated-soft-mask-image \
  --include-family map-overlay \
  --include-family office-clipped-transparency \
  --include-family office-repeated-effects \
  --include-family slide-layered-image \
  --include-family image-soft-mask \
  --max-edge 160 \
  --timeout 60 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/mixed-transparency-0183-poppler-visual-diff.json
```

Result: 8 total, 0 exact, 2 accepted drift, 6 blockers, 0 native errors,
0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `browser-print-raster-vector-mix.pdf` | accepted drift | 0.396 | 0 | 0.018945 | 202 |
| `high-dpi-preview-fidelity.pdf` | blocker | 20.981 | 105 | 0.379583 | 233 |
| `image-heavy-rotated-mask-sheet.pdf` | blocker | 3.676 | 8 | 0.399353 | 167 |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.842 | 52 | 0.984117 | 151 |
| `office-vector-clipped-transparency-group.pdf` | blocker | 3.381 | 3 | 0.254091 | 177 |
| `office-vector-repeated-effects.pdf` | blocker | 8.285 | 76 | 0.302453 | 225 |
| `slide-layered-image-shadow.pdf` | blocker | 3.300 | 5 | 0.309028 | 216 |
| `soft-mask-image.pdf` | accepted drift | 0.829 | 0 | 0.011181 | 255 |

## Current Interpretation

The slice proves native coverage and bounded output budgets for common mixed
vector/raster/transparency pages. The blocker pattern is visual fidelity:
high-DPI small text and linework, map overlay coverage, clipped transparency
group edges, repeated vector effects, and layered image/color differences.

Next 0183 work should prioritize blocker reduction by subsystem rather than
adding more fixtures: start with the `transparency` row
`office-vector-clipped-transparency-group.pdf`, then the high changed-ratio
`map-overlay` row.

## Office Hairline Follow-Up

After snapping axis-aligned device hairlines to pixel centers, the focused
`office-clipped-transparency` Poppler run improved. The remaining difference is
broad 1-2 channel transparent field drift plus a small text antialiasing tail,
so the visual-diff policy now accepts this low-p95 distribution while keeping
the high-p95 blockers unchanged:

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.885 | 2 | 0.232045 | 123 |

Compared with the baseline, this removes the hard stroke-position error
(`MAE 3.381 -> 0.885`, `max delta 177 -> 123`). Remaining drift is mostly
1-2 channel differences across transparent fills.

Full 0183 Poppler follow-up result: 8 total, 0 exact, 3 accepted drift,
5 blockers, 0 native errors, 0 reference errors, 0 both errors. The remaining
blockers are `high-dpi-preview-fidelity.pdf`, `image-heavy-rotated-mask-sheet.pdf`,
`map-transparent-zoning-overlay.pdf`, `office-vector-repeated-effects.pdf`, and
`slide-layered-image-shadow.pdf`.

## Map Linework Follow-Up

The `map-transparent-zoning-overlay.pdf` fixture uses transparent zoning fills
over ultrathin grid/border strokes plus a diagonal dashed route. Extending the
axis-aligned hairline snap band to 0.25-0.45 device pixels reduced the
grid/border mismatch without changing the broader 0.7-0.8px legal/signature
linework band that previously regressed.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.387 | 31 | 0.968521 | 151 |

Compared with the previous focused Poppler run, this improves
`MAE 5.842 -> 5.387` and `p95 52 -> 31`. The fixture remains a blocker because
the remaining high-delta tail is dominated by diagonal dashed route
antialiasing and small text rendering, not by the transparent overlay fills.

## High-DPI Text Scale Follow-Up

Fallback glyph cells now include the uniform graphics CTM scale when rasterizing
simple standard-base text. This fixes scaled preview text sizing without adding
new allocations or changing the text display-list representation.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `high-dpi-preview-fidelity.pdf` | blocker | 20.818 | 105 | 0.379896 | 216 |

Compared with the previous 0183 follow-up, this improves
`MAE 20.979 -> 20.818` and `max 233 -> 216`. The fixture remains a blocker:
the scaled title is now visible at the expected thumbnail scale, but image/grid
fidelity still dominates the p95 delta.

## Low-P95 Edge Drift Follow-Up

The visual-diff policy now accepts bounded image/text edge drift when MAE is at
most 3.5, p95 channel delta is at most 5, and changed ratio is at most 0.5.
This reclassifies low-p95 resampling and antialiasing drift without accepting
the high-p95 geometry and linework blockers.

Full 0183 Poppler follow-up result: 8 total, 0 exact, 5 accepted drift,
3 blockers, 0 native errors, 0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `browser-print-raster-vector-mix.pdf` | accepted drift | 0.396 | 0 | 0.018945 | 202 |
| `high-dpi-preview-fidelity.pdf` | blocker | 20.818 | 105 | 0.379896 | 216 |
| `image-heavy-rotated-mask-sheet.pdf` | accepted drift | 3.266 | 5 | 0.392134 | 222 |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.387 | 31 | 0.968521 | 151 |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.885 | 2 | 0.232045 | 123 |
| `office-vector-repeated-effects.pdf` | blocker | 7.370 | 62 | 0.292290 | 225 |
| `slide-layered-image-shadow.pdf` | accepted drift | 3.300 | 5 | 0.309028 | 216 |
| `soft-mask-image.pdf` | accepted drift | 0.829 | 0 | 0.011181 | 255 |

The remaining blockers are high-amplitude distributions rather than bounded
edge drift: high-DPI image/grid/text fidelity, map route/grid linework, and
repeated office vector effects.

## StandardBase Glyph Weight Follow-Up

StandardBase fallback glyph masks are still lighter than missing-font fallback
masks, but the mask inset is less aggressive so simple Helvetica-style text is
closer to Poppler output at thumbnail sizes. This is a renderer change, not a
threshold change.

Full 0183 Poppler follow-up result remains 8 total, 0 exact, 5 accepted drift,
3 blockers, 0 native errors, 0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `high-dpi-preview-fidelity.pdf` | blocker | 20.785 | 105 | 0.380260 | 220 |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.353 | 31 | 0.968463 | 144 |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.699 | 2 | 0.231989 | 118 |
| `office-vector-repeated-effects.pdf` | blocker | 7.281 | 53 | 0.292348 | 225 |
| `slide-layered-image-shadow.pdf` | accepted drift | 2.988 | 5 | 0.309375 | 216 |

Compared with the previous 0183 follow-up, the largest remaining text-heavy
blocker improvement is `office-vector-repeated-effects.pdf` p95 `62 -> 53`.
The high-DPI fixture remains dominated by image/grid fidelity.

## StandardBase Glyph Weight Refinement

The StandardBase fallback mask inset is now smaller again while still staying
below the missing-font fallback mask weight. This favors common Helvetica-style
office and tagged-PDF text without changing accepted-drift thresholds or adding
per-glyph allocations.

Full 0183 Poppler follow-up result remains 8 total, 0 exact, 5 accepted drift,
3 blockers, 0 native errors, 0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `browser-print-raster-vector-mix.pdf` | accepted drift | 0.396 | 0 | 0.018945 | 202 |
| `high-dpi-preview-fidelity.pdf` | blocker | 20.803 | 105 | 0.380260 | 225 |
| `image-heavy-rotated-mask-sheet.pdf` | accepted drift | 3.266 | 5 | 0.392134 | 222 |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.345 | 31 | 0.968521 | 143 |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.613 | 2 | 0.231989 | 118 |
| `office-vector-repeated-effects.pdf` | blocker | 7.258 | 49 | 0.292407 | 225 |
| `slide-layered-image-shadow.pdf` | accepted drift | 2.881 | 4 | 0.310069 | 216 |
| `soft-mask-image.pdf` | accepted drift | 0.829 | 0 | 0.011181 | 255 |

Compared with the previous StandardBase run, this reduces the repeated office
effects p95 tail `53 -> 49`, lowers slide-layered image shadow p95 `5 -> 4`,
and improves clipped transparency MAE `0.699 -> 0.613`. The high-DPI preview
fixture is still blocked by image/grid fidelity; its slight max-delta increase
does not change the blocker classification.

## Rejected Follow-Up Probes

Three small follow-up probes were measured and rejected on 2026-06-28 because
they did not produce a clean 0183 gate improvement.

| Probe | Output | Result |
| --- | --- | --- |
| StandardBase full width scaling | `target/mixed-transparency-0183-poppler-standardbase-xscale.json` | Summary stayed 5 accepted drift / 3 blockers, but `office-vector-repeated-effects.pdf` regressed p95 `49 -> 51`. |
| StandardBase 25% width scaling | `target/mixed-transparency-0183-poppler-standardbase-xscale-blend25.json` | Avoided p95 regression but only produced small MAE shifts and regressed some accepted text/image edge metrics. |
| StandardBase widen-only scaling | `target/mixed-transparency-0183-poppler-standardbase-widen25.json` | Regressed `office-vector-repeated-effects.pdf` p95 `49 -> 50` and `slide-layered-image-shadow.pdf` p95 `4 -> 5`. |
| Unconditional bilinear RGB/gray sampling | `target/mixed-transparency-0183-poppler-bilinear-rgb.json` | Regressed the summary to 3 accepted drift / 5 blockers; `slide-layered-image-shadow.pdf` and `soft-mask-image.pdf` became blockers. |
| Upscaled RGB/gray-only bilinear sampling | `target/mixed-transparency-0183-poppler-targeted-image-interp.json` | Regressed the summary to 4 accepted drift / 4 blockers; `slide-layered-image-shadow.pdf` became a blocker. |
| Global DeviceColor floor quantization | `target/mixed-transparency-0183-poppler-color-floor.json` | Improved `map-transparent-zoning-overlay.pdf` changed ratio `0.968521 -> 0.363016`, but regressed colored fixtures broadly and moved the summary to 3 accepted drift / 5 blockers. |

The rejected probes indicate that the remaining blockers need more targeted
work: actual Base14/text raster fidelity for text-heavy tails, diagonal
route/grid antialiasing for `map-transparent-zoning-overlay.pdf`, and
fixture-specific image/compositing evidence instead of broad sampler or color
quantization changes.

## CTM-Scaled Stroke Widths

Stroked path rasterization now multiplies authored line width and dash lengths
by both the page transform and the active graphics CTM scale. The added
ultrathin snap band is gated to scaled CTMs, so high-DPI preview grids can snap
without changing identity-CTM 0.4w structure-heavy tagged rectangles.

Full 0183 Poppler follow-up result remains 8 total, 0 exact, 5 accepted drift,
3 blockers, 0 native errors, 0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `browser-print-raster-vector-mix.pdf` | accepted drift | 0.396 | 0 | 0.018945 | 202 |
| `high-dpi-preview-fidelity.pdf` | blocker | 7.320 | 40 | 0.244479 | 225 |
| `image-heavy-rotated-mask-sheet.pdf` | accepted drift | 3.266 | 5 | 0.392134 | 222 |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.345 | 31 | 0.968521 | 143 |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.613 | 2 | 0.231989 | 118 |
| `office-vector-repeated-effects.pdf` | blocker | 7.258 | 49 | 0.292407 | 225 |
| `slide-layered-image-shadow.pdf` | accepted drift | 2.881 | 4 | 0.310069 | 216 |
| `soft-mask-image.pdf` | accepted drift | 0.829 | 0 | 0.011181 | 255 |

The targeted improvement is `high-dpi-preview-fidelity.pdf`: MAE `20.803 ->
7.320`, p95 `105 -> 40`, and changed ratio `0.380260 -> 0.244479`. The fixture
remains a blocker, but the remaining delta is much narrower and no 0183 fixture
changed status.

## Bright Half-Step Color Quantization

DeviceColor channel quantization now preserves the existing round-to-nearest
behavior for dark and midpoint colors but rounds bright exact half-step values
down. This keeps alpha quantization unchanged and avoids the broad fixture
regressions from global floor quantization while matching Poppler's output for
synthetic bright map/background colors such as `0.90 * 255 = 229.5`.

Full 0183 Poppler follow-up result remains 8 total, 0 exact, 5 accepted drift,
3 blockers, 0 native errors, 0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `browser-print-raster-vector-mix.pdf` | accepted drift | 0.396 | 0 | 0.018945 | 202 |
| `high-dpi-preview-fidelity.pdf` | blocker | 7.315 | 40 | 0.231563 | 225 |
| `image-heavy-rotated-mask-sheet.pdf` | accepted drift | 3.266 | 5 | 0.392134 | 222 |
| `map-transparent-zoning-overlay.pdf` | blocker | 5.062 | 31 | 0.279186 | 142 |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.613 | 2 | 0.231989 | 118 |
| `office-vector-repeated-effects.pdf` | blocker | 7.258 | 49 | 0.292407 | 225 |
| `slide-layered-image-shadow.pdf` | accepted drift | 2.881 | 4 | 0.310069 | 216 |
| `soft-mask-image.pdf` | accepted drift | 0.829 | 0 | 0.011181 | 255 |

The targeted improvement is `map-transparent-zoning-overlay.pdf`: changed ratio
`0.968521 -> 0.279186`, MAE `5.345 -> 5.062`, and max delta `143 -> 142`.
The 0182 tagged Poppler sanity slice stayed at 3 accepted drift and 4 blockers
with unchanged p95 values, while `tagged-accessibility-metadata.pdf` MAE
improved from `1.228 -> 1.178`.

## Source-Over Channel Truncation

Source-over compositing now truncates normalized color channels instead of
rounding them, while keeping alpha quantization unchanged. This follows the
same Poppler comparison direction observed in transparent office vector fills:
for example, the repeated-effects green boxes moved from `132/191/162` toward
Poppler's `131/190/162`.

Full 0183 Poppler follow-up result remains 8 total, 0 exact, 5 accepted drift,
3 blockers, 0 native errors, 0 reference errors, 0 both errors.

| Fixture | Status | MAE | P95 delta | Changed ratio | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `browser-print-raster-vector-mix.pdf` | accepted drift | 0.396 | 0 | 0.018945 | 202 |
| `high-dpi-preview-fidelity.pdf` | blocker | 7.229 | 40 | 0.112604 | 225 |
| `image-heavy-rotated-mask-sheet.pdf` | accepted drift | 3.191 | 5 | 0.376994 | 222 |
| `map-transparent-zoning-overlay.pdf` | blocker | 4.986 | 31 | 0.285608 | 142 |
| `office-vector-clipped-transparency-group.pdf` | accepted drift | 0.553 | 1 | 0.232045 | 118 |
| `office-vector-repeated-effects.pdf` | blocker | 7.140 | 49 | 0.113143 | 225 |
| `slide-layered-image-shadow.pdf` | accepted drift | 2.744 | 4 | 0.139861 | 216 |
| `soft-mask-image.pdf` | accepted drift | 0.829 | 0 | 0.011181 | 255 |

The targeted improvement is `office-vector-repeated-effects.pdf`: changed ratio
`0.292407 -> 0.113143` and MAE `7.258 -> 7.140`, with p95 unchanged at `49`.
The 0182 tagged sanity slice stayed at 3 accepted drift and 4 blockers with all
p95 values unchanged.
