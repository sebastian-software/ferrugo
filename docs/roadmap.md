# Roadmap

Status: historical roadmap, not an active implementation plan.
Date: 2026-06-24.

This roadmap records the original phase direction. Operational truth now lives
in readiness reports, policies, ADRs, and backlogs. Completed milestone
planning files were retired after their decisions and evidence were promoted to
durable documentation.

## Phase 0: Thumbnail Probe Definition

Goal: make thumbnail generation measurable before implementation choices become
expensive.

- Use `docs/plans/phase-0-decisions.md` as the decision baseline.
- Document MIT/Apache-2.0 as the license intent and attribution policy.
- Build PDFium from source with V8 and XFA disabled, AGG enabled, and Skia
  disabled.
- Measure binary size, cold start, first-page render time, thumbnail render
  time, and memory high-water mark.
- Create a local Rust CLI/library probe for single-page thumbnails.
- Use a backend-neutral Rust API facade so the PDFium backend does not define
  the public shape.
- Support page index default `0`, max edge default `1024`, timeout default
  `5s`, RGBA test output, and PNG artifact output.
- Use a serialized PDFium backend for the first probe.
- Create a fixture policy: generated PDFs in Git, curated real-world corpus
  outside Git.
- Defer npm, Node-API, prebuilt binaries, bundled PDFium, and distribution
  policy until after Phase 0 measurements.

Exit criteria:

- The cut-down PDFium build flags and measured outputs are documented.
- A local CLI can render PNG thumbnails from generated fixtures.
- RGBA output exists for differential comparison.
- The backend-neutral Rust API facade is sketched.
- Fixture policy is documented.
- The project license intent is explicit.
- npm, Node-API, prebuilt binaries, and PDFium bundling are explicitly deferred.

## Phase 1: Syntax And Object Model

Goal: load real PDFs into a safe object graph.

- Byte scanner and primitive parser.
- Indirect objects and references.
- Classic xref tables.
- Xref streams.
- Object streams.
- Trailer dictionaries.
- Basic repair mode for damaged xref data.
- Stream dictionary handling and filter dispatch.
- Typed errors with source offsets where available.

Exit criteria:

- The parser can enumerate pages and basic metadata for a mixed fixture set.
- Malformed inputs fail with typed errors rather than panics.
- Fuzzing exists for primitive parsing and object loading.

## Phase 2: Content Interpretation

Goal: convert page content streams into a display list.

- Resource resolution.
- Graphics state stack.
- Current transformation matrix handling.
- Path construction operators.
- Fill and stroke state.
- Clipping.
- Image XObject placement.
- Basic text state and text showing operators.
- Form XObject recursion with depth limits.

Exit criteria:

- Simple vector, image, and text PDFs produce inspectable display lists.
- Operator coverage is tracked in tests.
- Resource recursion and stream expansion are budgeted.

## Phase 3: Raster Rendering

Goal: produce useful RGBA page bitmaps.

- Raster device abstraction.
- Path fill and stroke rasterization.
- Antialiasing.
- Image sampling and interpolation.
- Basic alpha blending.
- Page transforms for scale, rotation, crop boxes, and backgrounds.
- Pixel comparison against PDFium for simple fixtures.

Exit criteria:

- Generated fixtures render to bitmaps with stable dimensions and tolerances.
- A command-line render tool can write PNG output.
- Pixel tests run in CI for a small stable fixture set.

## Phase 4: Fonts, Text, And Images

Goal: cover the parts that dominate real-world rendering quality.

- Embedded TrueType/OpenType/CFF font loading.
- Type1 and CID font strategy.
- CMaps and encodings.
- Glyph mapping and fallback.
- Text extraction baseline.
- DCT/JPEG, JPX/JPEG 2000, CCITT, JBIG2 strategy.
- Color spaces beyond DeviceRGB.

Exit criteria:

- Real-world office, browser, invoice, scanned, and vector PDFs render
  recognizably.
- Font regressions have reduced fixtures.
- Text extraction has a basic compatibility suite.

## Phase 5: Public APIs

Goal: expose the engine without freezing internals too early.

- Stable Rust API for document loading, page inspection, and rendering.
- Optional C ABI or FPDF-like compatibility facade for tests and integrations.
- Initial Node-API package with async rendering.
- TypeScript definitions and examples.
- Error taxonomy shared across Rust and Node.

Exit criteria:

- A Node user can open a PDF buffer and render a page to RGBA or PNG.
- Rust examples and Node examples share the same core behavior.
- The API can be versioned without exposing parser internals.

## Phase 6: Hardening

Goal: move from promising renderer to dependable engine.

- Larger public corpus.
- OSS-Fuzz or equivalent continuous fuzzing.
- Sandboxed or budgeted image/font/codecs where needed.
- More color management.
- Transparency groups, blend modes, soft masks, and patterns.
- Annotation rendering.
- Forms.
- Incremental loading.
- Performance profiling and tiling.

Exit criteria:

- The engine has documented compatibility limits.
- Crashes are treated as security bugs.
- Performance and memory regressions are tracked.

## First Spike Recommendation

Start with a two-week spike focused on thumbnail generation:

1. Build PDFium from source with V8 and XFA disabled, using the default AGG
   renderer.
2. Build a tiny CLI that calls PDFium and writes page PNGs for fixtures.
3. Measure binary size, cold start, render time, and memory for thumbnail-sized
   output.
4. Create a Rust facade trait for thumbnail backends so the PDFium backend does
   not define the public API.
5. Define the golden-test format that later Rust-native modules must satisfy.

That spike gives the project a test spine before large porting work begins. It
does not include npm packaging, Node-API bindings, prebuilt binaries, or a Cargo
workspace for the full Rust-native renderer.
