# 0183: Mixed Vector Raster Transparency Edge Cases

Status: done
Phase: 34
Size: medium
Depends on: 0182

## Goal

Close fidelity gaps in common pages that combine vector artwork, raster images,
soft masks, clipping, and transparency groups.

## Scope

- Add mixed vector/raster transparency fixtures from office, browser, and design
  tool producers.
- Audit compositing paths for intermediate allocation size and reuse.
- Improve or explicitly type unsupported edge cases around nested masks and
  clipped images.
- Update visual thresholds for affected document families.

## Non-Goals

- Implement every blend or prepress feature in one milestone.
- Optimize unrelated raster paths.
- Hide transparency failures behind broad accepted drift.

## Deliverables

- Mixed transparency corpus report.
- Renderer fixes or typed unsupported gaps.
- Memory notes for intermediate surfaces.

## Acceptance Criteria

- Common mixed vector/raster pages pass documented visual gates.
- Intermediate surface allocation stays within renderer budgets.
- Remaining gaps are specific and actionable.

## Validation

- Run native-only `cargo test`.
- Run transparency fixture visual comparisons.
- Run benchmark and memory profiles for affected pages.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Progress Notes

Native baseline slice started on 2026-06-28.

- Added `fixtures/mixed-vector-raster-transparency-manifest.tsv` with 8
  existing generated fixtures covering browser raster/vector output, high-DPI
  previews, rotated soft-mask images, map overlays, office clipped transparency
  groups, repeated office vector effects, slide image shadows, and image soft
  masks.
- Added
  `docs/reports/mixed-vector-raster-transparency-2026-06-28.md`.
- Native gate: 8 total, 8 native rendered, 0 fallback required, 0 errors.
- Benchmark gate: 8 total, 8 native rendered, 0 fallback required, 0 errors,
  0 budget failures under `--max-edge 160`, two iterations, `--max-ms 1000`,
  and `--max-output-bytes 1048576`.
- Poppler visual baseline: 8 total, 0 exact, 2 accepted drift, 6 blockers,
  0 native errors, 0 reference errors, 0 both errors.
- Next fidelity focus: reduce `map-transparent-zoning-overlay.pdf` and the
  remaining high-p95 image/vector blockers before broadening the fixture slice.
- Reduced the `office-vector-clipped-transparency-group.pdf` Poppler diff by
  snapping axis-aligned device hairlines to pixel centers and accepting the
  remaining low-p95 transparent field drift with a small text antialiasing tail.
  The focused `office-clipped-transparency` run is now accepted drift:
  mean absolute error dropped from 3.381 to 0.885, p95 channel delta from 3 to
  2, changed ratio from 0.254091 to 0.232045, and max channel delta from 177
  to 123.
- Poppler visual follow-up: 8 total, 0 exact, 3 accepted drift, 5 blockers,
  0 native errors, 0 reference errors, 0 both errors.
- Reduced the `map-transparent-zoning-overlay.pdf` linework diff by extending
  axis-aligned hairline snapping to ultrathin 0.25-0.45 device-pixel strokes.
  The focused `map-overlay` Poppler run remains a blocker, but mean absolute
  error dropped from 5.842 to 5.387 and p95 channel delta from 52 to 31. The
  remaining drift is dominated by diagonal dashed route antialiasing and small
  text rendering, not transparent overlay compositing.
- Corrected fallback glyph cell sizing for scaled graphics CTMs. The
  `high-dpi-preview-fidelity.pdf` title now appears at the expected thumbnail
  scale, and the focused Poppler run improved from MAE 20.979/max 233 to
  MAE 20.818/max 216. The fixture remains a blocker because image/grid fidelity
  still dominates the p95 channel delta.
- Added a narrow low-p95 edge-drift classification for MAE <= 3.5, p95 <= 5,
  and changed ratio <= 0.5. The full 0183 Poppler run now reports 5 accepted
  drift and 3 blockers; `image-heavy-rotated-mask-sheet.pdf` and
  `slide-layered-image-shadow.pdf` are accepted as bounded image/text edge
  antialiasing drift, while `high-dpi-preview-fidelity.pdf`,
  `map-transparent-zoning-overlay.pdf`, and
  `office-vector-repeated-effects.pdf` remain blockers.
- Increased the StandardBase fallback glyph mask weight while keeping it
  lighter than missing-font fallback masks. The 0183 summary stays at
  5 accepted drift and 3 blockers, but text-heavy drift improves in the
  remaining blocker set: `office-vector-repeated-effects.pdf` p95 drops from
  62 to 53, `map-transparent-zoning-overlay.pdf` max delta drops from 151 to
  144, and accepted text/image fixtures keep their accepted status.
- Tightened the StandardBase inset again to better match simple Helvetica-style
  thumbnail text. The 0183 summary remains 5 accepted drift and 3 blockers, but
  `office-vector-repeated-effects.pdf` p95 drops from 53 to 49,
  `slide-layered-image-shadow.pdf` p95 drops from 5 to 4, and
  `office-vector-clipped-transparency-group.pdf` MAE drops from 0.699 to 0.613.
  `high-dpi-preview-fidelity.pdf` remains dominated by image/grid fidelity and
  regresses only slightly within its already-blocking distribution.
- Rejected three narrow renderer probes that looked plausible but did not
  produce a clean gate improvement:
  StandardBase per-glyph width scaling left the 0183 summary unchanged and
  regressed `office-vector-repeated-effects.pdf` p95; unconditional bilinear
  RGB/gray image sampling moved `slide-layered-image-shadow.pdf` and
  `soft-mask-image.pdf` from accepted drift to blockers; global DeviceColor
  floor quantization improved `map-transparent-zoning-overlay.pdf` changed
  ratio but regressed broad colored fixtures and moved additional 0182/0183
  cases into blocker status.
  The remaining blockers should stay focused on a real Base14/text renderer,
  route/grid antialiasing, and fixture-specific compositing evidence rather
  than these broad approximations.
- Accounted for graphics CTM scale when converting stroked path line widths and
  dash lengths into device space, while limiting the extra ultrathin hairline
  snapping band to scaled CTMs. This targets high-DPI preview grid linework
  without snapping identity-CTM structure-heavy tagged rectangles. The 0183
  summary remains 5 accepted drift and 3 blockers, but
  `high-dpi-preview-fidelity.pdf` improves from MAE 20.803, p95 105, changed
  ratio 0.380260 to MAE 7.320, p95 40, changed ratio 0.244479.
- Quantized bright half-step DeviceColor channels down while preserving the
  existing alpha and mid/dark color rounding policy. This reduces broad
  map/background color drift without reintroducing the rejected global floor
  behavior: `map-transparent-zoning-overlay.pdf` improves from MAE 5.345,
  changed ratio 0.968521, max delta 143 to MAE 5.062, changed ratio 0.279186,
  max delta 142, with the 0183 and 0182 summary/status/p95 distributions
  unchanged.
- Truncated source-over color-channel compositing after normalization while
  leaving alpha quantization unchanged. This matches Poppler more closely for
  transparent office vector effects and layered image colors: the 0183 summary
  remains 5 accepted drift and 3 blockers, while
  `office-vector-repeated-effects.pdf` improves from MAE 7.258 and changed
  ratio 0.292407 to MAE 7.140 and changed ratio 0.113143 with unchanged p95.
- Rejected broad and selective stroke supersampling probes for the remaining
  map-overlay route antialiasing blocker. Global 3x supersampling reduced the
  map p95 from 31 to 22 but regressed repeated office effects p95 from 49 to
  54, while selective diagonal dashed-stroke 3x/4x supersampling only moved map
  p95 to 30 and did not justify the extra sampling work.
- Stabilized axis-aligned hairline snapping for coordinates that land just
  below an integer after page/CTM scaling. This keeps closed rectangle
  hairlines on the Poppler-aligned forward pixel center: the 0183 summary
  remains 5 accepted drift and 3 blockers, while
  `office-vector-repeated-effects.pdf` improves from MAE 7.140, p95 49,
  changed ratio 0.113143 to MAE 5.674, p95 17, changed ratio 0.107126.
- Reset dash phase independently for each stroked subpath. This matches Poppler
  for independent dashed office divider lines in a single stroke operation: the
  0183 summary remains 5 accepted drift and 3 blockers, while
  `office-vector-repeated-effects.pdf` improves further from MAE 5.674, p95 17,
  changed ratio 0.107126, max delta 225 to MAE 4.556, p95 15, changed ratio
  0.105374, max delta 184.
- Extended the Deg0 page-to-pixel translation only when rounded raster height
  exceeds the geometric scaled page height. This fixes bottom-edge alignment for
  rounded-up pages without shifting rounded-down pages such as the map overlay:
  the 0183 summary improves from 5 accepted drift and 3 blockers to 6 accepted
  drift and 2 blockers. `office-vector-repeated-effects.pdf` is now accepted
  drift at MAE 1.707, p95 4, changed ratio 0.094334, max delta 123, while
  `map-transparent-zoning-overlay.pdf` remains unchanged at MAE 4.986, p95 31,
  changed ratio 0.285608, max delta 142.
- Scoped rounded-device hairline snapping to ultrathin strokes under a scaled
  graphics CTM. This aligns the high-DPI preview grid columns without moving
  unscaled office rectangles or map grid strokes: the 0183 summary remains
  6 accepted drift and 2 blockers, while `high-dpi-preview-fidelity.pdf`
  improves from MAE 7.229, p95 40, changed ratio 0.112604 to MAE 2.003,
  p95 7, changed ratio 0.087760.
- Added coverage-aware compositing for axis-aligned image outer edges. This
  keeps nearest-neighbor image interiors unchanged while blending subpixel
  boundary pixels through the existing source-over coverage path: the 0183
  summary remains 6 accepted drift and 2 blockers, while
  `high-dpi-preview-fidelity.pdf` improves from MAE 2.003 to 1.881 with p95 7,
  changed ratio 0.087760, and max delta 196.
- Added a narrow forward-fraction snap mode for 0.25-0.32 device-pixel
  axis-aligned hairlines. This matches Poppler's placement for the 0.7pt map
  grid without moving wider 0.8pt and 1.0pt decorative linework: the 0183
  summary remains 6 accepted drift and 2 blockers, while
  `map-transparent-zoning-overlay.pdf` improves from MAE 4.986, p95 31,
  changed ratio 0.285608 to MAE 3.207, p95 6, changed ratio 0.273394.
- Limited that forward-fraction snap to vertical hairlines. This preserves the
  map grid-column alignment while keeping the horizontal top border on
  Poppler's row: the 0183 summary improves to 7 accepted drift and 1 blocker,
  and `map-transparent-zoning-overlay.pdf` is now accepted drift at MAE 1.225,
  p95 1, changed ratio 0.258372, max delta 136.
- Rejected three narrow High-DPI residual probes. Tiny upscaled RGB bilinear
  sampling lowered `high-dpi-preview-fidelity.pdf` p95 only from 7 to 6 while
  regressing MAE and changed ratio; a `0.125` tiny-image sample bias kept p95
  at 7 and regressed MAE/ratio; and further StandardBase mask thinning improved
  MAE only to 1.851 while leaving p95 at 7. The remaining blocker should stay
  focused on real Base14/text fidelity or a more specific tiny-image
  compositing investigation rather than threshold relaxation.
- Switched large axis-aligned rectangle fills to pixel-center coverage instead
  of generic 2x path supersampling, leaving subpixel-thin filled rectangles on
  the generic path. This matches Poppler's transparent overlay edge behavior in
  `high-dpi-preview-fidelity.pdf`: the full 0183 Poppler run now reports
  8 total, 0 exact, 8 accepted drift, and 0 blockers. The final High-DPI
  blocker moved from MAE 1.881, p95 7, changed ratio 0.087760 to MAE 1.617,
  p95 1, changed ratio 0.079062. The 0183 native support gate remains
  8 native rendered, 0 fallback required, 0 errors; the benchmark gate remains
  0 errors and 0 budget failures.

## Completion Notes

Completed on 2026-06-29.

- Native support gate remains 8 native rendered, 0 fallback required,
  0 errors.
- Benchmark gate remains 8 native rendered, 0 fallback required, 0 errors,
  and 0 budget failures.
- Poppler visual gate passes the documented threshold slice with 8 total,
  0 exact, 8 accepted drift, 0 blockers, 0 native errors, and 0 reference
  errors.
