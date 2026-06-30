# PDFium Performance Findings, 2026-06-30

Status: working research note.

This document studies the current PDFium source to identify performance
patterns that may explain why PDFium is fast on common documents. It is not a
proposal to transliterate PDFium into Rust. Ferrugo should remain Rust-first,
but PDFium is still the most useful behavioral and performance reference.

## Scope

The source snapshot was read from PDFium `main` on 2026-06-30, commit
`53a619295f667480446775baa4c0cc0aa1dc2724`.

The first pass focused on:

- page/object rendering dispatch;
- the classic CPU graphics device backed by AGG;
- path, stroke, rectangle, and clip handling;
- image loading, rendering, and page-local image cache behavior;
- text rendering and glyph batching;
- progressive rendering and cache trimming.

The phrase "black magic" is useful shorthand, but the code mostly shows a
stack of mundane, well-placed decisions: avoid work before rasterization, use
specialized device operations when the shape is simple, cache decoded resources
with explicit budgets, and keep the inner loops tightly specialized.

## High-Level Source Map

- `core/fpdfapi/render/cpdf_renderstatus.cpp` owns object-level dispatch:
  visibility checks, object bounds clipping, transparency routing, and
  path/text/image/form dispatch.
- `core/fpdfapi/render/cpdf_progressiverenderer.cpp` owns progressive
  traversal through layer objects, pause points, and cache trimming after image
  work.
- `core/fxge/cfx_renderdevice.cpp` is the higher-level device abstraction. It
  decides whether a path can become a rectangle fill, native text, glyph bitmap
  rendering, text path rendering, or driver-level path rendering.
- `core/fxge/agg/cfx_agg_devicedriver.cpp` is the classic CPU raster backend.
  It uses AGG scanline rasterization, clip masks, span compositing, and several
  direct bitmap/rectangle operations.
- `core/fpdfapi/page/cpdf_pageimagecache.cpp`,
  `core/fpdfapi/page/cpdf_dib.cpp`, and
  `core/fpdfapi/render/cpdf_imagerenderer.cpp` cover image decode, image cache,
  target-size-aware loading, masks, and device image compositing.

## Finding 1: PDFium Culls Before It Rasterizes

PDFium does not send every page object into the rasterizer. In
`CPDF_RenderStatus::RenderObjectList`, objects are skipped when inactive or
outside the current clip rectangle before `RenderSingleObject()` is called.
`RenderSingleObject()` then checks visibility options, applies the object's
clip path, routes transparency, and only then dispatches by object type.

Why this matters:

- object bounding boxes are cheap compared with path flattening, stroke
  expansion, glyph loading, image decode, or span compositing;
- this prevents off-screen or clipped-out objects from even reaching the
  device driver;
- progressive rendering uses a similar object bounds gate before continuing an
  object.

Relevant source:

- object skip before render:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_renderstatus.cpp#223
- object dispatch:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_renderstatus.cpp#240
- progressive object bounds gate:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_progressiverenderer.cpp#77

Ferrugo implication:

- Keep the existing device-bounds culling, but move more decisions earlier:
  display-list item construction should retain enough conservative bounds to
  skip flattening and route building before pixel work starts.
- Add profile counters for "objects considered", "objects culled by device
  bounds", "paths flattened", and "paths rasterized". If many items survive
  culling but later produce zero spans, the bounds are too loose or the raster
  route is too late.

## Finding 2: PDFium Has A Device-Level Fast Path Layer

PDFium's `CFX_RenderDevice::DrawPath()` checks for several cheap cases before
falling through to the device driver's generic path rasterizer:

- a two-point no-stroke-alpha path can be rendered as a cosmetic line;
- filled rectangles without antialiased rectangle handling can become
  `FillRect()`;
- zero-area paths get separate handling;
- combined fill-and-stroke can go through driver support or a specialized
  fallback path.

This is important because the high-level renderer does not need to know every
backend trick. It asks the device to draw a path; the device chooses the
cheapest correct route for the current backend and render options.

Relevant source:

- path fast-path gate:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/cfx_renderdevice.cpp#687
- rectangle conversion:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/cfx_renderdevice.cpp#709
- device fill rectangle:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#1252

Ferrugo implication:

- Keep shape classification close to the raster device boundary. The display
  list can carry intent, but the final route should account for transform,
  clip, alpha, antialiasing, and target pixel format.
- The next vector plan should stop adding tiny branches inside an already hot
  predicate. Instead, add a route-classification report: how many paths become
  filled rect, stroked axis line, simple stroke, generic stroke, clipped fill,
  text path, or image composite.

## Finding 3: The CPU Rasterizer Is Scanline/Span-Oriented

The AGG backend does not manually test every path against every pixel in a
general-purpose PDF loop. It builds AGG path storage, feeds
`agg::rasterizer_scanline_aa`, and renders scanline spans. The custom
`CFX_AggRenderer` then composites spans into the destination bitmap.

The inner compositing code is specialized by:

- destination format and bytes per pixel;
- alpha output versus non-alpha output;
- full coverage versus partial coverage;
- clip mask presence;
- RGB byte order;
- backdrop/group-knockout presence.

The constructor selects a composite span function once, and the render loop
then walks only spans that AGG emits. The span loop still has branches, but it
starts from a much smaller representation than a full pixel grid.

Relevant source:

- AGG render call:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#1178
- span renderer fields and selected composite function:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#507
- span loop:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#842
- specialized compositing branches:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#520

Ferrugo implication:

- Our current vector hotspot still spends heavily in row-bucketed stroke ranges
  and `blend_pixel`. That points toward a span-oriented renderer or at least a
  route that emits contiguous covered spans before blending.
- The first step does not have to be a full AGG-equivalent. A pragmatic next
  experiment is a `StrokeSpan`/`FillSpan` intermediate for simple strokes and
  rectangles, with one compositing pass per span instead of per candidate
  sample.
- Add counters for produced spans, average span length, covered pixels, skipped
  pixels, and blend calls. Without that, we are still optimizing branches
  inside a loop whose shape may be wrong.

## Finding 4: PDFium Trades Invisible Micro-Detail For Bounded Work

`RasterizeStroke()` applies a notable policy: if a dash cycle becomes smaller
than `0.1` device pixels, PDFium renders the stroke as solid. The code comments
explicitly frame this as avoiding performance issues while preserving visual
fidelity, because gaps that small are imperceptible.

The same function also clamps stroke width to at least one device-unit
equivalent, maps PDF caps/joins to AGG caps/joins, and splits transform handling
so stroke expansion can operate with a normalized scale.

Relevant source:

- stroke setup and minimum width:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#299
- dash-cycle threshold:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#338
- transformed stroke split:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#1229

Ferrugo implication:

- We should add an explicit "device-pixel policy" section to the renderer
  docs: minimum hairline width, microscopic dash collapse, coordinate clamps,
  and acceptable thumbnail-scale approximations.
- This should not be hidden as a speed hack. It is a fidelity/performance
  policy with visual tests.
- The vector-stress fixture should include microscopic dash patterns so the
  benchmark can measure whether this policy removes pathological route work.

## Finding 5: PDFium Uses Rectangular Clip Shortcuts Before Clip Masks

When setting a fill clip, the AGG driver first asks whether the path is a
rectangle after transform. If it is, PDFium intersects the clip region with the
rectangle and returns. Only non-rectangular clips allocate/build a rasterizer
and create a mask.

For fills, `FillRect()` also intersects against the current clip box and exits
early if the draw rectangle is empty. If a clip mask exists it composites
through the mask; otherwise it uses direct rectangle compositing.

Relevant source:

- rectangular clip path shortcut:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#1111
- clip mask generation for complex clips:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#1087
- direct rectangle fill:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/agg/cfx_agg_devicedriver.cpp#1252

Ferrugo implication:

- We already gained from clip-bound intersection on technical hatch fixtures.
  The next refinement should classify clip shape, not just clip bounds:
  rectangular clip, small convex clip, complex path clip, image/mask clip.
- Rectangular clips should become bounds-only constraints when possible.
  Complex clip masks should be allocated lazily and sized to the intersected
  path bounds, not to the full page.

## Finding 6: Image Performance Comes From Cache + Target Size + Continuation

PDFium has a page-local image cache keyed by image stream. It tracks cache
burden, access time, and can purge old entries when a cache limit is exceeded.
It also validates cached images against the requested maximum size. If an image
was decoded for a smaller target size than the current render needs, the cache
entry is not blindly reused.

The image renderer asks the loader to use the render device dimensions as
`max_size_required`, which lets decode/cache decisions know the target scale.
For huge images, non-halftone rendering flips on bilinear interpolation.

Relevant source:

- cache trimming:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/page/cpdf_pageimagecache.cpp#119
- cache lookup and insert:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/page/cpdf_pageimagecache.cpp#170
- cached bitmap validity by requested size:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/page/cpdf_pageimagecache.cpp#268
- image load request with device-size requirement:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_imagerenderer.cpp#68
- huge-image resampling policy:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_imagerenderer.cpp#419

Ferrugo implication:

- A bounded request/session image cache should become a first-class benchmark
  option. It must be explicit, tenant-safe, and visible in reports.
- The cache key should include stream identity, decode parameters, color
  transform, mask/soft-mask state, and target-size class.
- The scan/image track should prioritize target-aware decode and avoiding
  full-resolution RGBA expansion when rendering thumbnails.

## Finding 7: Text Rendering Is Batched Around Glyph Bitmaps

For normal text, PDFium tries native/device text where allowed, otherwise
loads glyph bitmaps, builds a combined glyph bounding box, clips that box to
the device clip, then draws into a temporary bitmap/mask before compositing it
back to the device. Very large text can be routed through text paths.

This is not just a fidelity choice. It avoids per-glyph device writes when a
run can be batched into one temporary surface, and it keeps empty clipped text
from allocating after the glyph bounding box is intersected.

Relevant source:

- text mode dispatch:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_renderstatus.cpp#817
- normal text route:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_renderstatus.cpp#919
- glyph bitmap loading and run bounds:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/cfx_renderdevice.cpp#1236
- temporary text bitmap/mask composite:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fxge/cfx_renderdevice.cpp#1271

Ferrugo implication:

- Text performance should not be treated as a sequence of independent glyph
  draws. Add a text-run raster path that computes run bounds, clips once,
  reuses scratch bitmap/mask storage, and composites once per run where
  possible.
- Glyph cache design should be separated from text-run scratch reuse. The first
  cache is long-lived within the document/session; the second is short-lived
  per render.

## Finding 8: Progressive Rendering Also Serves Performance And Memory

PDFium's progressive renderer processes objects in bounded chunks, yields at
pause points, handles images as continuations, and trims the image cache when
limited-cache mode is active. It also continues parsing the page object holder
only when needed.

Even when Ferrugo is server-side and not UI-progressive, this shape is useful:
it creates natural budget checkpoints, cancellation points, and memory trim
points.

Relevant source:

- progressive render loop:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_progressiverenderer.cpp#46
- cache trim after images:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_progressiverenderer.cpp#99
- render context cache trim after a layer:
  https://pdfium.googlesource.com/pdfium/+/53a619295f667480446775baa4c0cc0aa1dc2724/core/fpdfapi/render/cpdf_rendercontext.cpp#61

Ferrugo implication:

- Use the same concept for server rendering: object budget checkpoints,
  cancellation, and explicit cache trim points after large image/form work.
- Report these as timings/counters, not just as control flow. That will expose
  whether long-tail renders are dominated by one giant object or by many small
  objects.

## What This Means For The Current Ferrugo Performance Work

The recent vector optimization loop already showed that tiny inner-branch
changes are not enough. Several candidates improved less than 5%, were neutral,
or regressed. PDFium's source points to a stronger direction:

1. Move more decisions out of the hottest predicate loops.
2. Classify render routes before raster work begins.
3. Emit spans/ranges as the primary unit of compositing.
4. Keep rectangle and rectangular-clip paths as first-class routes.
5. Collapse visually irrelevant microscopic dash detail.
6. Add explicit session/request caches for decoded shared resources.
7. Batch text into runs and scratch surfaces instead of independent glyph
   writes.
8. Keep budget checkpoints and cache trim points visible in benchmark reports.

## Proposed Next Experiments

These are intentionally phrased as experiments, not assumed wins.

### Experiment A: Route Classification Trace

Add a native trace section that records counts for:

- display items considered;
- display items culled by device bounds;
- paths flattened;
- paths routed as fill rect;
- paths routed as rectangular clip;
- paths routed as simple stroke;
- paths routed as generic stroke;
- spans emitted;
- blend calls;
- clip-mask allocations and dimensions;
- decoded images and cache hits/misses.

Acceptance:

- no output change;
- trace fields are opt-in and redacted like existing timing attribution;
- the report identifies which route dominates `vector-stress`,
  `technical-hatch-clipping`, and one scan/image fixture.

### Experiment B: Span-Oriented Simple Stroke Path

For a narrow subset of strokes, emit row spans before blending:

- butt or square caps first;
- no dash or dash already collapsed to solid;
- no active complex clip mask;
- conservative device bounds already known;
- scalar fallback remains the existing raster path.

Acceptance:

- at least 5-10% repeated improvement on the target vector subset as part of a
  cumulative span track, or documented rejection;
- unchanged output dimensions and no fallback changes;
- counters show fewer blend calls or longer contiguous spans.

### Experiment C: Microscopic Dash Collapse Policy

Implement a device-pixel dash policy equivalent in spirit to PDFium's
sub-`0.1px` dash-cycle collapse, guarded by visual fixtures.

Acceptance:

- visible policy documented in renderer docs;
- fixtures cover tiny dash cycles and normal dash cycles;
- benchmark proves a reduction in dash segment count or raster route work;
- no broad visual claim without differential review.

### Experiment D: Bounded Image Session Cache

Add an explicit request/session cache for decoded image resources.

Acceptance:

- no global cache;
- cache budget and hit/miss counters appear in benchmark output;
- cache key includes target-size class and mask/color parameters;
- scan/image fixture shows either speed gain, memory gain, or a clean rejection.

## Open Questions

- Does Ferrugo's current generated corpus contain enough microscopic dash,
  repeated-image, and dense-text-run cases to exercise the same classes of
  optimization that PDFium carries?
- Are our current `PathDisplayItem` bounds tight enough to make early route
  classification reliable, or do we need route-specific bounds?
- Should the first span renderer live inside `ferrugo-render` as a small
  internal abstraction, or should it be introduced as a separate raster module
  with stricter invariants?
- Which fidelity policy should govern thumbnail-scale approximations such as
  microscopic dash collapse?

## Bottom Line

PDFium's speed does not appear to come from one exotic trick. It comes from
layering cheap decisions before expensive work and from keeping expensive work
in specialized scanline, glyph, image, and rectangle paths. For Ferrugo, the
highest-leverage next step is not another tiny predicate rewrite. It is route
classification plus span-oriented compositing evidence, followed by one narrow
span fast path if the trace proves the route is hot enough.
