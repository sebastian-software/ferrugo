# Raster Architecture Gap Analysis, 2026-07-02

Status: working research note.

This document closes the loop between the two earlier research notes
([PDFium findings](2026-06-30-pdfium-performance-findings.md) and
[cross-renderer findings](2026-06-30-cross-renderer-performance-findings.md))
and Ferrugo's actual rasterizer implementation. The earlier notes cataloged
patterns: early culling, device-level fast paths, span compositing, bounded
caches, target-aware image decode, and progressive execution. This note asks
the harder question: which structural differences explain the remaining
order-of-magnitude gap against PDFium, and which of the reference engines'
core ideas are worth adapting first.

The short answer: the reference engines do not merely have better-tuned inner
loops. They rasterize with a fundamentally cheaper algorithm class. Ferrugo is
a point-sampling rasterizer; PDFium, MuPDF, Poppler/Splash, Skia, and FreeType
are all edge-based analytic coverage rasterizers. Most of the other famous
tricks (span blitters, stroke-to-fill, banded parallelism) only become
available after that switch.

## Reuse Boundary

Same rule as the earlier notes: this is source-informed research, not a
porting plan. No implementation code, comments, constants, tables, or tests
from PDFium, MuPDF, Poppler, Skia, tiny-skia, stb_truetype, or font-rs may be
copied. The algorithm class described below (signed-area coverage
accumulation) is published, decades old, and implemented independently many
times (libart, FreeType, AGG, stb_truetype, font-rs, tiny-skia). Ferrugo
should implement it from the published descriptions and our own tests, with
independently measured thresholds.

## Evidence: Where Ferrugo Actually Loses Time

Three bodies of evidence agree.

**The benchmark gap is concentrated in vector/text-heavy families.** The
archived 0078 smoke run (same corpus, `max_edge=160`, shared thumbnail facade;
see [benchmarks](../benchmarks.md)) showed:

| Family | Ferrugo mean ms | PDFium mean ms | Ratio |
| --- | ---: | ---: | ---: |
| `report` | 267.832 | 0.581 | ~460x |
| `presentation` | 15.452 | 0.382 | ~40x |
| `browser-print` | 38.607 | 0.665 | ~58x |
| `mixed-layout` | 17.910 | 2.338 | ~8x |
| `scan` | 1.293 | 1.166 | ~1x |

The `scan` family (image blit dominated) is already at parity. The gap lives
where path and text rasterization dominate.

**Release profiles put nearly all time in per-pixel geometry predicates and
per-pixel blending.** The 2026-06-30 release sample on `vector-stress`
(recorded in the
[performance working plan](../plans/2026-06-29-performance-optimization-working-plan.md))
has `rasterize_row_bucketed_stroke_ranges`, `blend_pixel`,
`rasterize_span_covered_stroke_ranges`, `stroke_path`, and
`axis_stroke_raster_spans` as the top symbols. Content tokenization and object
parsing are an order of magnitude smaller. The bottleneck is not parsing, not
decoding, and not I/O. It is the shape of the raster loop.

**Micro-optimizations inside that loop have plateaued.** The working plan
records a string of rejected candidates (pre-sort checks in
`merge_pixel_ranges`, butt-cap predicate specialization, sampled-blend
dispatch, direct byte writes), each moving the needle less than the noise
floor. Meanwhile the one algorithmic change in this area — clipped pixel-bounds
culling in milestone 0096 — delivered 15.4x on the same fixture. The pattern is
consistent: work-avoidance wins, instruction-shaving does not. The remaining
work to avoid is per-pixel geometry testing itself.

## What Ferrugo's Rasterizer Is Today

Verified against `crates/ferrugo-render/src/lib.rs` on this branch.

- **Fills are point-sampled.** `fill_path` (lib.rs:11072) iterates device
  pixels inside clipped bounds and, per pixel, tests `supersample²` sample
  points (default 2x2 = 4) with `point_in_path` (lib.rs:14037), which runs
  ray-casting (`point_in_polygon_even_odd`, lib.rs:14057) or winding
  (`polygon_winding`, lib.rs:14072) against every flattened subpath. Cost is
  roughly `bounds_pixels × samples² × edges`, softened only by bounds culling
  and an axis-aligned-rectangle fast path (`fill_axis_aligned_rect_path`,
  lib.rs:11124).
- **Strokes are point-sampled distance tests.** `stroke_path` (lib.rs:11724)
  routes between four strategies (axis-aligned spans, single-line spans,
  row-bucketed candidates, span-covered ranges), but the non-axis-aligned core
  is still `point_in_stroke` (lib.rs:14121) / `point_in_join` (lib.rs:14279):
  per sample, distance-to-segment against candidate lines plus cap/join
  geometry. The row buckets and span cursors reduce the candidate set; they do
  not change the per-sample cost model.
- **Anti-aliasing is 2x2 supersampling.** That yields 5 coverage levels per
  pixel. Reference engines produce 256 analytic coverage levels in a single
  pass, so the current design pays 4x the geometry tests for visibly coarser
  antialiasing.
- **Curve flattening is fixed-count.** `flatten_path_segments` (lib.rs:10951)
  subdivides every cubic into 16 uniform segments (`cubic_point`,
  lib.rs:11060) regardless of curve size or device scale. At thumbnail scale
  (`max_edge` 160–320) a typical glyph- or chart-sized curve needs 2–4
  segments for sub-pixel error; we generate 16, and every extra segment is
  another edge in every point-in-path test that touches the path.
- **Clipping is per-sample point testing.** `point_in_active_clips`
  (lib.rs:14009) walks the clip stack per sample; axis-aligned rect clips
  short-circuit to `point_in_rect`, everything else re-runs point-in-path per
  clip. There are no rasterized clip masks; clip-bounds intersection narrows
  pixel bounds only.
- **Blending is a per-pixel call with runtime dispatch.** `blend_pixel`
  (lib.rs:14840) is invoked per covered pixel, re-checks blend mode and
  opacity fast paths per call, and writes 4 bytes at a time. There are no
  row/span blitters selected once per draw item.
- **Text and images are comparatively healthy.** Glyph raster work goes
  through a bounded per-pass `GlyphBitmapCache` (lib.rs:3030, 256 entries,
  oldest-first eviction); images have axis-aligned opaque fast paths and
  sample caches (`draw_image`, lib.rs:14970; `ImageSampleCache`,
  lib.rs:15355); decode already uses `zune-jpeg`, which is in the
  libjpeg-turbo performance class. The `scan` parity above confirms this
  track.
- **The display list is a usable IR.** Typed items with graphics state and
  bounds (`DisplayItem`, lib.rs:1276) already exist, which matters below: it
  is exactly the structure MuPDF uses as its multithreading and banding
  boundary.
- **Everything is single-threaded.** No rayon/threads in the render path,
  matching the current plan decision to parallelize across pages first.

## The Core Ideas Of The Fast Engines

### Idea 1: Analytic Cell-Based Coverage Rasterization

This is the closest thing to actual "black magic," and it is shared by every
fast CPU 2D engine: libart → FreeType `ftgrays` → AGG (vendored in PDFium as
`third_party/agg23`) → stb_truetype → font-rs → tiny-skia. MuPDF's active-edge
scan converter and Splash's XPath scanner are cousins from the same family.

The algorithm:

- Path coordinates are converted once into fixed-point subpixel space (AGG:
  24.8, `poly_subpixel_shift = 8`).
- Each edge is walked once, depositing into per-pixel cells two signed values:
  `cover` (vertical crossing contribution) and `area` (exact partial-pixel
  signed area).
- A scanline sweep sorts/buckets cells by row, keeps a running sum of `cover`
  across each row, and emits (a) exact antialiased coverage at edge pixels
  from `area`, and (b) full-coverage interior spans between edges from the
  running sum.
- Fill rule (nonzero/even-odd) is applied to the accumulated integer, then
  clamped to 0–255 coverage.

Properties that matter for us:

- Cost is `O(edges × scanlines_touched + covered_cells)`, independent of how
  many pixels the bounding box has and independent of subpath count per test.
  Interior pixels of a filled region cost nothing per pixel except the blit.
- Antialiasing is exact (256 levels) in one pass. No supersampling, so the 4x
  sample multiplier disappears while quality improves.
- The winding accumulation gives nonzero and even-odd for free — no ray
  casting, no per-sample winding loops.
- The output is naturally span-shaped, which unlocks Idea 4.
- The core is small. font-rs demonstrates the dense-accumulation-buffer
  variant (deposit signed area differences into an `f32` row buffer, integrate
  with a prefix sum) in roughly a hundred lines of safe scalar Rust, with an
  optional SIMD prefix-sum later. That variant maps cleanly onto our
  MSRV/no-unsafe-hot-path rules and onto the plan's "SIMD only after a
  row-level compositor exists" decision.

Relevant sources:

- AGG cell/cover/area rasterizer:
  https://agg.sourceforge.net/antigrain.com/__code/include/agg_rasterizer_scanline_aa.h.html
- AGG fixed-point basics:
  https://agg.sourceforge.net/antigrain.com/__code/include/agg_basics.h.html
- AGG scanline/span containers (packed vs unpacked tradeoff):
  https://agg.sourceforge.net/antigrain.com/doc/scanlines/scanlines.agdoc.html
- Sean Barrett's explanation of the signed-area algorithm and its lineage:
  https://nothings.org/gamedev/rasterize/
- Raph Levien, "Inside the fastest font renderer in the world" (font-rs):
  https://medium.com/@raphlinus/inside-the-fastest-font-renderer-in-the-world-75ae5270c445
- PDFium's vendored AGG:
  https://pdfium.googlesource.com/pdfium/+/refs/heads/master/third_party/agg23/
- MuPDF active-edge scan conversion:
  https://github.com/ArtifexSoftware/mupdf/blob/master/source/fitz/draw-edge.c

Ferrugo implication: this replaces `point_in_path`, `point_in_polygon_even_odd`,
`polygon_winding`, per-sample supersampling, and most of the per-pixel clip
testing in one structural change. It is the single highest-leverage item in
this document.

### Idea 2: Strokes Become Fill Outlines

None of the reference engines test pixel-to-segment distance at raster time.
All of them expand the stroke geometry once per draw into an outline polygon
(caps, joins, and dashes included) and feed it to the same fill rasterizer:

- AGG: `conv_stroke` converts a path into its stroked contour before
  rasterization.
- MuPDF: `fz_flatten_stroke_path` flattens the stroked path directly into the
  same global edge list the fill path uses.
- Skia: `FillPathWithPaint` / `getFillPath` turns stroke+paint into a fill
  path; the raster backend strokes nothing directly.

Relevant sources:

- https://agg.sourceforge.net/antigrain.com/doc/index.html
- https://github.com/ArtifexSoftware/mupdf/blob/master/source/fitz/draw-path.c
- https://fiddle.skia.org/c/@FillPathWithPaint

Ferrugo implication: `point_in_stroke`, `point_in_join`, the prepared join
sides, the row buckets, the join buckets, and the span-cursor machinery — the
entire family currently at the top of every `vector-stress` profile — become
one geometry pass (stroke outliner) plus the shared fill rasterizer. Dash
handling stays a geometry-stage concern, where the existing microscopic-dash
collapse policy idea from the PDFium note also naturally lives. This is only
worth doing after (or together with) Idea 1; feeding stroke outlines into the
current point-sampling fill would trade one per-pixel predicate for another.

### Idea 3: Curve Flattening With Device-Space Tolerance

Reference engines flatten curves adaptively against an error tolerance in
device pixels, so segment count tracks the transform:

- AGG subdivides recursively until the deviation is below a distance tolerance
  derived from `approximation_scale`, which is set from the current transform.
- MuPDF computes `flatness = 0.3f / expansion` from the CTM and passes it into
  fill/stroke flattening — about a third of a device pixel of error at any
  zoom.
- kurbo (Rust) implements Levien's analytic flattening, which computes the
  required segment count directly from an error functional instead of
  recursing, and is the published state of the art for a Rust implementation.

Relevant sources:

- https://agg.sourceforge.net/antigrain.com/research/adaptive_bezier/
- https://github.com/ArtifexSoftware/mupdf/blob/master/source/fitz/draw-path.c
- https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html

Ferrugo implication: our fixed 16-segment subdivision is wrong in both
directions — wasteful at thumbnail scale (where most of our workloads live)
and imprecise under large zoom. Unlike Ideas 1–2 this is a small, independent
change to `flatten_path_segments` that pays off immediately: fewer edges mean
cheaper point-in-path tests today and a smaller edge list for the coverage
rasterizer tomorrow. It needs a documented device-space tolerance policy and
visual fixtures, like the dash-collapse policy.

### Idea 4: Coverage Spans Feed Blitters Selected Once Per Draw

Downstream of the rasterizer, the fast engines never make a per-pixel decision
twice:

- AGG emits scanlines of spans; PDFium's `CFX_AggRenderer` selects one
  composite-span function per draw (by pixel format, alpha, clip mask, RGB
  order) and then only walks spans.
- Skia's scan converter hands coverage runs to a blitter; the default
  `SkRasterPipelineBlitter` chains single-purpose SIMD stages (load coverage →
  color → load dst → srcover → store) assembled once per draw, with `highp`
  f32 and `lowp` u16 variants. tiny-skia is the same architecture in Rust
  with function-pointer pipelines instead of a JIT, and lands within
  20–100% of Skia on x86-64 — evidence the architecture, not C++ or a JIT, is
  what matters.

Relevant sources:

- https://skia.googlesource.com/skia/+/6887dcf153bf/docs/architecture/CPU.md
- https://github.com/linebender/tiny-skia
- Span compositing in PDFium's AGG driver: see Finding 3 in
  [the PDFium note](2026-06-30-pdfium-performance-findings.md).

Ferrugo implication: once Idea 1 produces `(y, x0, x1, coverage)` spans, the
current `blend_pixel`-per-pixel model should become: classify the draw once
(blend mode, source opacity, pixel format, clip state), pick a row blitter,
then run it over spans. Interior spans of opaque fills become `memcpy`-class
row writes. This is also the correct substrate for the SIMD work the 0159
evaluation deferred: vectorizing a span blitter is worthwhile; vectorizing
per-pixel predicate calls was not.

### Idea 5: Glyphs Are Cached Raster Assets With Quantized Keys

The engines treat rendered glyph bitmaps as document-lifetime cached assets
with carefully quantized keys:

- MuPDF's glyph store keys on font, glyph id, the four matrix components
  (fixed-point), antialias level, and a *size-dependent* subpixel position:
  glyphs under 24px quantize position to 1/8 px, 24–47px to 1/4 px, 48px and
  larger to whole pixels. Bigger glyphs need less positional resolution, so
  hit rates stay high without visible error. Oversized glyphs bypass the cache.
- PDFium's `CFX_GlyphCache` keys on the text matrix (scaled fixed-point),
  destination width, antialias mode, and font flags, per face, for the
  document lifetime — repeated body text across pages is a pure cache hit.
- FreeType ships a whole cache subsystem (`FTC_Manager`, `FTC_SBitCache`) that
  packs small bitmaps densely because per-node overhead dominates at glyph
  sizes.

Relevant sources:

- https://github.com/ArtifexSoftware/mupdf/blob/master/source/fitz/draw-glyph.c
- https://pdfium.googlesource.com/pdfium/+/09b419242/core/fxge/cfx_facecache.cpp
- http://freetype.org/freetype2/docs/reference/ft2-cache_subsystem.html

Ferrugo implication: `GlyphBitmapCache` already stores geometry-only bitmaps
with quantized cell size, which is the right shape. The gaps are lifetime
(per-rasterization-pass today; should be request/session scoped alongside the
shared-resource cache so multi-page batches reuse glyphs), keying (no
size-tiered subpixel quantization), eviction (oldest-first, no byte
accounting), and coverage (real embedded-font glyph rasters, once the coverage
rasterizer makes glyph filling cheap and exact). Type 3 glyph programs should
additionally cache their rendered result under the strict conditions the
cross-renderer note already describes.

### Idea 6: Parse Lazily, Cache Decoded Resources For The Document

Beyond the render loop, PDFium's speed on typical office documents also comes
from the document layer: pages parse on demand, linearized files get a
first-page fast path, and `CPDF_DocPageData` caches parsed fonts, color
spaces, and decoded images document-wide. Fonts in particular are parsed once
and shared by every page.

Relevant sources:

- https://pdfium.googlesource.com/pdfium/+/master/core/fpdfapi/parser/cpdf_parser.cpp
- Findings 6–8 in [the PDFium note](2026-06-30-pdfium-performance-findings.md).

Ferrugo implication: milestone 0188 (shared document/page-tree loading per
request) and the linearized first-page loader already move in this direction.
The remaining step is request-scoped sharing of *decoded* resources — font
programs, glyph outlines, decoded images, ICC transforms — with the byte
accounting and eviction counters the repo's cache policy demands. This mostly
matters for multi-page batch workloads, which is exactly the product shape.

### Idea 7: Band Rendering And Display-List Parallelism

MuPDF renders through a display list precisely so that rasterization can be
split into horizontal bands and replayed concurrently: `mutool draw -B
bandheight -T threads` replays one immutable display list into N band pixmaps
on N threads, and peak raster memory drops to `width × bandheight`.

Relevant sources:

- https://mupdf.readthedocs.io/en/latest/cookbook/c/multi-threaded.html
- https://mupdf.readthedocs.io/en/latest/tools/mutool-draw.html

Ferrugo implication: our display list is already an immutable, replayable IR,
so banding is architecturally cheap once the rasterizer consumes scissored
bounds. The current plan defers inner parallelism in favor of page-level
parallelism — that decision stands. Record this as the designed-for later
stage: bands give both a parallelism unit and a raster-memory bound, which
aligns with the serverless memory budgets. Nothing in Ideas 1–4 should be
built in a way that assumes whole-page raster targets.

### Idea 8: Images — Mostly Already Adopted

The image track (target-aware decode hints, downsample plans, axis-aligned
opaque fast paths, sample caches, `zune-jpeg` decode) already matches the
reference-engine playbook, and the `scan` family benchmarks at parity with
PDFium. Remaining known deltas are policy-level, not architectural: bilinear
interpolation for heavily downscaled content (PDFium flips it on for huge
images) and the multi-step halving strategy PDF.js uses for extreme
downscales. Keep these on the existing image track; they are not part of the
core gap.

## Gap Summary

| Dimension | Reference engines | Ferrugo today | Consequence |
| --- | --- | --- | --- |
| Fill raster | Edge/cell coverage, one pass, 256 AA levels | Per-pixel 2x2 point sampling | Dominant cost on vector/text families; coarser AA |
| Stroke raster | Stroke-to-fill outline, shared fill machinery | Per-sample distance to segments/joins | Top profile symbols on `vector-stress` |
| Curve flattening | Adaptive, device-space tolerance | Fixed 16 segments per cubic | Excess edges at thumbnail scale, error under zoom |
| Clipping | Rect scissors + lazily built masks intersected with spans | Per-sample clip-stack point tests | Clip cost scales with samples, not with clip complexity |
| Compositing | Span blitters selected once per draw | `blend_pixel` per pixel with runtime dispatch | Per-pixel call and branch overhead; blocks useful SIMD |
| Glyph raster | Document-lifetime bitmap caches, quantized subpixel keys | Per-pass 256-entry cache, no subpixel tiers | Repeated text re-rasterizes across pages |
| Document layer | Lazy parse + document-wide decoded-resource caches | Shared parse per request (0188); decode still page-local | Multi-page batches redo decode work |
| Parallelism | Display-list banding across threads | Single-threaded page raster (by plan) | Acceptable now; keep banding-compatible |

## Adoption Roadmap

Ordered by leverage over effort. Each block follows the working plan's
operating rules: release-build matrix evidence, profiles before code, 10%
standalone or repeated 5–10% cumulative-track acceptance, protection set
neutral, no global caches, no copied code.

1. **Adaptive curve flattening (small, independent, do first).** Replace fixed
   16-segment subdivision with error-bounded flattening against a documented
   device-space tolerance. Add counters for segments emitted per curve and
   total edges per display item. Acceptance: fewer flattened segments on
   curve-heavy fixtures at thumbnail scale with neutral visual diff, and a
   measured win on at least one curve-heavy fixture.
2. **Coverage rasterizer prototype for fills (the structural centerpiece).**
   Implement a signed-area cell/accumulation rasterizer (font-rs-style dense
   row buffer is the simplest safe-Rust variant) behind an explicit raster
   route, initially for fill paths only. Integrate clip as scissored bounds
   for rect clips and as a rasterized coverage mask (multiplied per span) for
   complex clips, built lazily at intersected-bounds size. Acceptance: pixel
   output within the visual-oracle drift policy, large measured win on
   `report/vector` fixtures, and route counters showing which paths take the
   new route.
3. **Stroke-to-fill on top of the coverage rasterizer.** Implement a stroke
   outliner (caps, joins, miter limit, dashes, hairline minimum-width and
   dash-collapse policy) producing fill outlines. Retire the distance-predicate
   stroke routes once parity is proven, keeping them temporarily as a
   comparison oracle. Acceptance: `vector-stress` and technical-linework
   fixtures dominated by the fill route; stroke-predicate symbols disappear
   from release profiles.
4. **Span blitters.** Convert compositing to per-draw-selected row blitters
   over coverage spans: opaque-normal direct writes, source-over with
   coverage, blend-mode variants, clip-mask variants. This is also the gate
   for revisiting SIMD (0159 decision) with a real row-level compositor to
   vectorize. Acceptance: `blend_pixel`-class symbols replaced by span
   functions in profiles; measured win on fill-heavy fixtures.
5. **Glyph and resource cache lifetimes.** Promote the glyph bitmap cache to
   request/session scope with byte accounting, size-tiered subpixel
   quantization, and hit/miss/eviction counters; extend the 0188
   shared-document model to decoded fonts/images/ICC transforms under the
   existing cache policy. Acceptance: multi-page batch fixtures show decode
   and glyph-raster time amortized across pages.
6. **Banded rendering and inner parallelism (later wave, by existing plan
   decision).** Once the coverage rasterizer consumes scissored bounds, band
   the page raster for memory bounds first, threads second. Acceptance:
   documented raster-memory high-water reduction, then scheduler evidence.

Items 1 and 2 can start immediately and independently. Item 3 depends on 2.
Item 4 depends on 2. Items 5 and 6 are independent of each other and of 3–4
but sequenced after the raster core stabilizes to avoid optimizing a loop
that is about to be replaced.

## Why Not Keep Tuning The Current Loop

The current model's cost is approximately
`covered_bounds_pixels × samples² × candidate_geometry_tests`, and the last
five optimization candidates inside that product were rejected as noise-level.
The coverage model's cost is approximately
`edges + covered_cells + spans`, which for typical vector content is orders of
magnitude fewer operations, each simpler. PDFium's ~460x lead on the `report`
family is consistent with that complexity difference plus the 4x supersampling
multiplier — and cannot be closed by branch shaving inside the current
per-sample predicates. The 0096 experience (15.4x from bounds culling, <5%
from everything since) is the same lesson measured locally.

## What Not To Do

- Do not port AGG, ftgrays, or tiny-skia code. Implement from the published
  algorithm descriptions with our own tests and thresholds.
- Do not build the coverage rasterizer as a whole-page-only design; keep
  scissored/banded targets in the type signatures from day one.
- Do not start SIMD before span blitters exist (reaffirms the 0159 decision).
- Do not add document-lifetime or global caches outside the existing
  request/session cache policy; every new store needs budgets and counters.
- Do not claim public performance parity from single local runs; the
  benchmark-matrix rules in the working plan still apply.

## Open Questions

- Should the coverage rasterizer live inside `ferrugo-render` or as a new
  narrow crate (e.g. `ferrugo-raster`) with strict invariants and its own
  fuzz targets? The fuzzing setup and the 26k-line `lib.rs` argue for a crate.
- Dense accumulation buffer (font-rs style, simple, O(width) scratch per row
  band) versus sorted sparse cells (AGG style, better for very sparse tall
  paths)? The fixture corpus should decide; start with dense for simplicity.
- Which device-space flattening tolerance and AA gamma policy do we adopt as
  the documented rendering policy, and which fixtures gate them?
- How do text rendering modes (stroke text, clip text) map onto the
  stroke-to-fill and coverage-mask machinery — do they fall out naturally or
  need dedicated routes?
- When the coverage rasterizer lands, do the existing stroke-route counters
  and pixel-range structures get deleted, or kept behind a comparison oracle
  for one release?

## Bottom Line

The reference engines' "black magic" decomposes into one big idea and several
compounding ones. The big idea is analytic cell-based coverage rasterization
with stroke-to-fill on top — the shared core of FreeType, AGG/PDFium, MuPDF,
Splash, and Skia — which replaces Ferrugo's per-sample geometry predicates,
the supersampling multiplier, and per-sample clip tests in one structural
change, while improving antialiasing quality. The compounding ideas — adaptive
flattening, span blitters, quantized glyph caches, document-scoped decoded
resources, banded parallelism — each either require or are amplified by that
core. The recommended sequence is: adaptive flattening now, coverage-fill
prototype next, stroke-to-fill and span blitters on top, cache lifetimes and
banding after.
