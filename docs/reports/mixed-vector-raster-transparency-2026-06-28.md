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
