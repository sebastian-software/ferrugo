# Cross-Renderer Performance Findings, 2026-06-30

Status: working research note.

This document extends the PDFium performance pass with MuPDF, Poppler, and
PDF.js. The goal is to learn from mature renderers without copying source code.
Ferrugo should use these engines as performance references and design
inspiration, then implement Rust-native structures and algorithms from first
principles, benchmarks, PDF semantics, and our own tests.

## Reuse Boundary

This is source-informed research, not a porting plan.

- Do not copy implementation code, comments, constants, tables, or tests from
  MuPDF, Poppler, PDF.js, or PDFium.
- Treat hard-coded thresholds and cache sizes as observations, not as values to
  reuse. Ferrugo thresholds need independent measurement and visual tests.
- Prefer describing patterns at the architecture level: route classification,
  early culling, span emission, bounded caches, target-aware image decode, and
  progressive execution.
- When a future Ferrugo change is inspired by one of these engines, the commit
  should cite this document and the benchmark/profile evidence, not the source
  file as something being transliterated.

## Source Snapshots

- MuPDF: ArtifexSoftware/mupdf commit
  `1ca8f05d43928ba956fe20f27d8946af2466710d`, read from GitHub.
- Poppler: freedesktop.org/poppler commit
  `92a295fa290b0d122fef3d00e3276068914cea18`, read from GitLab.
- PDF.js: mozilla/pdf.js commit
  `614349086b034f2b02e8e3e2ec7f7376899476c7`, read from GitHub.

## Bottom Line

The engines differ in language, API surface, and product constraints, but the
performance story repeats:

- avoid work before rasterization, especially through page/object bounds and
  clip intersection;
- classify simple cases close to the device boundary;
- rasterize paths into spans or scanlines, not a general-purpose per-pixel PDF
  loop;
- reuse decoded resources and scratch surfaces with explicit budgets;
- downscale or crop images before expensive full-resolution work where the
  target transform allows it;
- make progressive execution and cancellation part of the render loop rather
  than an afterthought.

For Ferrugo, this strengthens the current direction: vector/report performance
should start with route classification and span-oriented drawing before SIMD or
micro-optimizing `blend_pixel`.

## MuPDF Findings

MuPDF's CPU renderer is organized around a draw device, a rasterizer, and
resource stores. The code reads like a system designed to keep expensive work
inside narrow bounds.

### Rasterizer Selection And Bounds Clipping

`fz_convert_rasterizer()` selects between rasterizer implementations based on
antialiasing configuration. Before conversion, it intersects rasterizer bounds
with the target pixmap and clip bounds. Empty intersections return before
pixel work starts.

Relevant source:

- rasterizer selection and bbox intersection:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-rasterize.c#L279-L310
- fill-path flattening and scissor-aware rasterizer reset:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-path.c#L308-L333
- stroke-path flattening with the same early-empty shape:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-path.c#L1607-L1627

Ferrugo implication:

- Make "empty after transform + clip" a first-class route outcome before
  flattening and before raster item allocation.
- Store counters for `empty_before_flatten`, `empty_after_flatten`,
  `flattened_paths`, and `rasterized_paths`. Without this split, a profile only
  tells us that path work is slow, not which gate is missing.

### Active-Edge Scan Conversion

MuPDF's edge rasterizer keeps active edges, walks scanlines, computes clipped
horizontal coverage, reuses alpha/delta buffers, and calls span blitters for
covered regions. The important idea is not the exact implementation; it is that
coverage is generated as row-local spans with clipping integrated into the scan
process.

Relevant source:

- active edge scan conversion and AA blitting:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-edge.c#L569-L752

Ferrugo implication:

- Add a span prototype for simple fills and strokes. The first useful target is
  not "all paths"; it is axis-aligned rectangles, hairlines, and simple
  transformed lines from the `report/vector` fixtures.
- Report average span length, produced spans, blended pixels, and skipped
  pixels. If spans are short and fragmented, the win is likely elsewhere.

### Target-Aware Image Decode

MuPDF carries target and subarea information into image decode. It can compute
target extents from the current transform, choose a subsampling factor, stream
only a subarea, skip margins, and subsample while reading.

Relevant source:

- image cache key and target extent computation:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/image.c#L390-L437
- subarea and subsampling stream setup:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/image.c#L500-L614
- decompression with subsampling and subarea handling:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/image.c#L617-L747

Ferrugo implication:

- The image track should avoid unconditional full RGBA expansion. Decode policy
  should know the target pixel size, visible subarea, interpolation mode, mask
  needs, and output format before allocating large buffers.
- Add benchmark fields for original image dimensions, target dimensions,
  decoded dimensions, decoded bytes, and mask scratch bytes.

### Bounded Stores, Glyphs, And Pattern Tiles

MuPDF uses explicit stores for reusable rendered or decoded data. The generic
store deduplicates by key, checks size limits, reaps space, and touches entries
for reuse. Glyphs and pattern tiles have their own keys and cache paths.

Relevant source:

- generic store insert, dedupe, space checks, and LRU touch:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/store.c#L440-L574
- glyph render/cache path with size guards and eviction:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-glyph.c#L279-L420
- tile pattern cache keyed by transform, document/resource identity, color, and
  shape:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-device.c#L2771-L3048

Ferrugo implication:

- Ferrugo should not add global caches, but request/session-local stores are
  likely necessary for repeated images, glyphs, and pattern tiles.
- Cache keys must include transform-relevant and color-relevant fields. A cache
  that is too broad is a correctness bug; a cache that is too narrow only wastes
  memory and time.
- Every store needs visible budget, hit/miss/eviction counters, and fixture
  evidence.

### Device Setup And Affine Image Fast Paths

MuPDF initializes a draw device with scissor state, rasterizer state, and scale
caches. Affine image drawing exits early for zero alpha, intersects the target
bbox with the scissor, maps screen to image space, selects a specialized paint
function, then loops through rows.

Relevant source:

- draw device setup and rasterizer allocation:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-device.c#L3166-L3274
- affine image draw dispatch and clipped row loops:
  https://github.com/ArtifexSoftware/mupdf/blob/1ca8f05d43928ba956fe20f27d8946af2466710d/source/fitz/draw-affine.c#L3898-L4123

Ferrugo implication:

- Keep the render-device boundary explicit. The high-level display list should
  describe work; the device should decide the cheap route after transform,
  scissor, alpha, interpolation, and pixel format are known.

## Poppler/Splash Findings

Poppler's Splash backend reinforces the same pattern from a different lineage:
PDF operator processing is separated from a raster backend that has strong
clip/span/image special cases.

### Operator Dispatch Is Table-Driven

`Gfx.cc` maps PDF operators to function handlers and argument constraints. This
keeps parsing/validation separate from output-device behavior, and it gives the
renderer a stable operator stream to consume.

Relevant source:

- operator table:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/poppler/Gfx.cc#L144-L225
- main operator loop with command profiling and abort checks:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/poppler/Gfx.cc#L619-L690

Ferrugo implication:

- Our content-stream layer should remain boring and measurable: decode
  operator, validate operands, append/render, count failures. Hot rendering
  should not depend on ad hoc string/operator branching.

### Clip Classification Avoids Pixel Work

Splash repeatedly asks whether a rectangle is outside, fully inside, or
partially clipped. Full-inside spans use cheaper paths; partial clips test
spans/pixels only where needed.

Relevant source:

- pixel and span drawing with clip handling:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/Splash.cc#L1347-L1485
- pattern fill bbox, clip classification, and span iteration:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/Splash.cc#L2364-L2485
- path scanner constructor clamping segment ranges to clip bounds:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/SplashXPathScanner.cc#L42-L118
- scanline intersections and span iteration:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/SplashXPathScanner.cc#L177-L329

Ferrugo implication:

- Add clip state classification to the raster context: `empty`, `full`,
  `rectangular`, and `mask/complex`.
- Keep separate fast paths for `full` and `rectangular` clips. Complex clip
  masks should not tax the common unclipped case.

### Narrow Stroke And Image Blit Routes

Splash has a distinct path for narrow strokes and separate image paths that
classify clipping before drawing. The image blit path can split work into
unclipped and clipped regions, avoiding per-pixel clip checks for the full
inside part.

Relevant source:

- stroke dispatch and narrow stroke path:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/Splash.cc#L1932-L2076
- transformed image-mask rectilinear branches:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/Splash.cc#L2747-L2819
- image blit clip splitting:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/splash/Splash.cc#L4987-L5121

Ferrugo implication:

- Hairlines and narrow strokes deserve their own measured route. They are too
  common in technical/report documents to leave inside the generic stroke path.
- Image compositing should split fully visible rows/spans from clipped rows
  before entering a mask-aware inner loop.

### Type 3 Glyph Cache Is Explicitly Sized

Splash has a Type 3 glyph cache with bounded associativity and data sizing. The
details are implementation-specific, but the idea matters: rendered glyph
programs are cacheable only under strict dimensions and memory budgets.

Relevant source:

- Type 3 cache setup:
  https://gitlab.freedesktop.org/poppler/poppler/-/blob/92a295fa290b0d122fef3d00e3276068914cea18/poppler/SplashOutputDev.cc#L1101-L1163

Ferrugo implication:

- Type 3 glyphs are mini content streams. Cache their rendered result only when
  the glyph size, transform class, color behavior, and memory budget make it
  safe.

## PDF.js Findings

PDF.js is not a native CPU renderer in the same sense as MuPDF or Splash, but
it is valuable because it makes streaming, dependency management, and
progressive rendering explicit.

### Operator Lists Are A Render IR

PDF.js builds an `OperatorList` with function IDs, argument arrays, dependency
markers, transferable buffers, and chunk flushing. It flushes large lists and
also tries to flush near natural boundaries such as restore/end-text.

Relevant source:

- operator-list chunking, dependencies, IR, and transferables:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/core/operator_list.js#L653-L810
- partial evaluator building operator lists with local image/color/gstate and
  pattern caches:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/core/evaluator.js#L1683-L1910

Ferrugo implication:

- Ferrugo's display list should be treated as a measurable IR, not just a
  temporary implementation detail. It needs stable item counts, byte estimates,
  dependency/resource IDs, and route classification fields.
- Chunking is useful even server-side for cancellation, timeouts, and memory
  high-water control.

### Execution Yields And Waits For Resources

Canvas execution stops when a dependency is not ready and yields after bounded
time/step windows when a continuation callback exists. This is browser-shaped,
but the same concept maps to server timeouts and cooperative cancellation.

Relevant source:

- canvas operator execution, dependency waiting, and time slicing:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/display/canvas.js#L618-L745

Ferrugo implication:

- Keep cancellation checks at display-list chunk boundaries and after large
  image/path operations. Avoid sprinkling checks inside every pixel loop unless
  profiles show long non-interruptible loops.

### Scratch Canvases And Bitmap Caches Are Local

PDF.js uses canvas-factory scratch surfaces, local pattern caches, bitmap maps,
and cleanup at end-of-drawing. It also pre-scales images in multiple half-size
steps and reuses ping-pong canvas entries instead of allocating a fresh surface
for every step.

Relevant source:

- image downscale steps and ping-pong scratch surfaces:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/display/canvas.js#L820-L930
- repeated mask/image bitmap cache:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/display/canvas.js#L930-L1088
- local/global image cache with page threshold and byte limit:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/core/image_utils.js#L199-L305

Ferrugo implication:

- Scratch buffers should be owned by a render session and reported as high-water
  memory. They should be reset/reused rather than hidden behind global state.
- Repeated-image caching should require evidence of reuse. PDF.js waits until
  an image appears across enough pages before global caching is worthwhile; the
  Ferrugo equivalent should be explicit and benchmark-visible.

### Type 3 And Font Paths Become Dependencies

PDF.js converts glyph paths and Type 3 glyph programs into reusable objects and
operator-list fragments. This makes glyph rendering a dependency/caching
problem, not just a text loop problem.

Relevant source:

- font path generation:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/core/evaluator.js#L4770-L4797
- Type 3 char procedure operator-list construction:
  https://github.com/mozilla/pdf.js/blob/614349086b034f2b02e8e3e2ec7f7376899476c7/src/core/evaluator.js#L4885-L4938

Ferrugo implication:

- The text roadmap should track font-path/glyph-program reuse separately from
  normal embedded font rendering. Type 3 can be very expensive and can also
  amplify clipping, transparency, and image behavior.

## Cross-Engine Patterns To Carry Forward

1. **Early outcome classification.** Each render item should become one of:
   culled, rect fill, hairline, simple stroke, generic stroke, image blit,
   image resample, text glyph, text path, pattern tile, form/group, or fallback.
2. **Clip state is a hot input.** `no clip`, `rect clip`, `full inside`,
   `partial`, and `complex mask` should be distinct routes, not flags checked
   in every inner loop.
3. **Spans beat candidate pixels.** Mature rasterizers produce coverage spans
   or scanline runs before compositing. Ferrugo should prototype this for the
   fixtures that currently spend time in per-sample stroke logic.
4. **Caches must be budgeted and local.** Glyphs, images, color spaces, pattern
   tiles, and scratch buffers need request/session lifetimes, clear keys, and
   hit/miss/eviction counters.
5. **Images need target context.** Decode/resample decisions should know the
   target transform, visible subarea, interpolation mode, mask/alpha needs, and
   output format before allocating the largest representation.
6. **Progressive rendering is an engine property.** Even server-side rendering
   benefits from chunk boundaries, cancellation checks, and visible time spent
   in parse/evaluate/raster/output phases.
7. **Fast paths belong near the device.** The high-level renderer should not
   predict every backend condition. The device/raster context should choose the
   cheapest correct route after final transform, clip, alpha, and pixel format
   are known.

## Recommended Ferrugo Experiments

These are ordered by expected leverage for the current `report/vector` gap.

1. Add route-classification counters for every display item.
   - Output: counts by route, culled count, fallback count, and time by route.
   - Acceptance: no visual change; benchmark report gains enough detail to say
     whether vector fixtures are slow because of too many generic paths,
     too-loose clips, or expensive blending.
2. Prototype a span path for axis-aligned rectangles and hairlines.
   - Output: `Span { y, x0, x1, coverage }` or equivalent internal shape for
     a narrow first route.
   - Acceptance: repeated improvement on vector fixtures, no regression on a
     small mixed-layout protection set.
3. Split clip routes in the raster context.
   - Output: separate `no_clip`, `rect_clip`, and `mask_clip` loops for
     rectangle fill and simple spans.
   - Acceptance: fewer per-pixel clip checks in profiles and stable output.
4. Add request-local scratch buffers and high-water reporting.
   - Output: reusable row/span/image scratch buffers, no global cache.
   - Acceptance: lower allocation count or scratch allocation bytes on the
     focused vector and image fixtures.
5. Add target-aware image instrumentation before optimizing decode.
   - Output: original dimensions, target dimensions, decoded dimensions,
     visible subarea, mask bytes, and final output bytes in benchmark reports.
   - Acceptance: identifies at least one scan/image fixture where full decode
     is wasteful enough to justify a decode change.
6. Add a bounded session resource store behind an explicit benchmark option.
   - Output: parsed resource/image/glyph cache with budget, hit/miss/eviction
     counters, and no hidden global state.
   - Acceptance: memory high-water stays bounded and repeated-resource fixtures
     show measurable wins.

## What Not To Do Yet

- Do not start with SIMD. The current evidence points to too much work reaching
  hot loops, not simply slow arithmetic inside the right loops.
- Do not add a broad cache. A cache without route counters and budget reporting
  will hide memory growth and may mask real algorithmic problems.
- Do not port one renderer's rasterizer structure line by line. The useful
  lesson is the shape of the pipeline, not the exact C/C++/JavaScript code.
- Do not publish performance claims against MuPDF/Poppler/PDFium until the
  benchmark matrix has repeated, versioned reference runs on the same host.

