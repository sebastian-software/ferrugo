# Thumbnail Generation Plan

Status: planning note.
Date: 2026-06-24.

## Goal

Generate reliable preview images from PDF files. The current product target is
thumbnail rendering, not full PDF viewing, editing, form handling, JavaScript
execution, or a full PDFium-compatible public API.

This changes the project shape. A thumbnail generator can start with a heavily
reduced feature set and still be useful:

- Open a PDF from bytes or a file path.
- Render one selected page per call, defaulting to page index `0`.
- Render at a bounded size.
- Return PNG for user-visible artifacts or raw RGBA for backend comparison.
- Fail predictably on encrypted, malformed, or unsupported PDFs.

Phase 0 decisions and defaults are centralized in
`docs/plans/phase-0-decisions.md`.

## Short Answer: Can PDFium Be Cut Down?

Yes. PDFium has GN build flags for excluding major optional features. The most
important ones for thumbnail generation are:

- `pdf_enable_v8 = false`: removes JavaScript support.
- `pdf_enable_xfa = false`: removes XFA form support. PDFium documents that XFA
  depends on JavaScript.
- `pdf_use_skia = false`: keeps the default AGG renderer path instead of the
  experimental Skia backend.
- `pdf_is_standalone = false`: avoids standalone test/tool targets in normal
  product builds.
- `pdf_is_complete_lib = true`: can produce a complete static library when used
  with `is_component_build = false`.

This does not make PDFium tiny. It still needs the parser, page model, renderer,
font handling, image codecs, color handling, memory/runtime support, and public
embedder API. But it removes large areas that are irrelevant to thumbnails.

## Recommended Strategy

Use a two-track plan.

### Track A: Source-Build Baseline With Cut-Down PDFium

Build and measure a small thumbnail CLI/library around a minimal PDFium
configuration. This gives immediate high-quality output and a behavior oracle
for any Rust implementation.

Initial config direction:

```gn
is_debug = false
is_component_build = false
pdf_enable_v8 = false
pdf_enable_xfa = false
pdf_use_skia = false
pdf_use_agg = true
pdf_is_standalone = false
pdf_is_complete_lib = true
clang_use_chrome_plugins = false
use_remoteexec = false
```

The runtime API should expose only:

- `render_thumbnail(input, page_index = 0, max_edge = 1024, background, format, timeout = 5s)`
- `page_count(input)`
- `page_size(input, page_index)`

Do not expose document editing, forms, JavaScript, annotation editing, or raw
PDFium handles in the product API.

The Phase 0 probe must measure:

- PDFium binary size,
- cold start,
- first-page render time,
- thumbnail render time at fixed output sizes,
- memory high-water mark.

### Track B: Rust-Native Replacement Surface

Design the Rust implementation around the same narrow thumbnail contract. The
first Rust-native milestone should not try to match PDFium broadly. It should
only render a controlled subset of PDFs well enough for preview images.

The Rust backend should be built Rust-first, not as a mechanical translation of
PDFium. PDFium defines the expected behavior; Rust defines the architecture,
ownership model, error model, and buffer handling. See
`docs/decisions/0001-rust-first-pdfium-guided-porting.md`.

The smallest useful Rust-native scope:

- Parse enough PDF structure to find pages and resources.
- Decode common streams: Flate, ASCIIHex, ASCII85, DCT/JPEG.
- Interpret basic page content: paths, fills, strokes, images, simple text,
  clipping, transformations.
- Render to RGBA at thumbnail sizes.
- Compare output against the cut-down PDFium baseline.

Implementation rules for this track:

- Start safe by default.
- Use `#![forbid(unsafe_code)]` in parser, object-model, content, and public API
  crates.
- Use safe slice copying and stride-aware buffers before raw pointer copying.
- Keep unsafe isolated to FFI, codecs, SIMD, or measured pixel-buffer hotspots.
- Optimize after the thumbnail fixture loop is correct and profiled.

The Phase 0 Rust-native work is limited to the backend-neutral facade shape and
comparison contract. It does not require a Rust parser or renderer
implementation yet.

Defer:

- JavaScript.
- XFA.
- Interactive forms.
- Digital signatures.
- Editing and saving.
- Full annotation semantics.
- Full color management.
- Exact rendering parity for every edge case.

## Why Not Port MuPDF First?

MuPDF is attractive for a thumbnail renderer because its core engine is compact,
fast, and focused. But the open-source license is AGPL, and a direct port would
need careful derivative-work handling. That makes it a worse primary base if the
goal is a permissively usable Rust/Node package.

MuPDF should still be studied for architecture:

- compact document/rendering pipeline,
- low-level rendering primitives,
- display list model,
- memory/error discipline,
- CLI ergonomics for raster output.

But avoid using MuPDF code structure as the direct implementation blueprint
unless the license strategy is decided first.

## Why Not Port Poppler First?

Poppler is mature and useful as a comparison renderer, but it is less aligned
with this goal:

- C++ codebase with long desktop/Linux history,
- GPL-oriented licensing,
- broader desktop document-viewer assumptions,
- less direct fit for Chrome-like Node/server thumbnails.

Use Poppler as a third oracle for disputed PDFs, not as the primary port target.

## Initial Architecture

```text
Node / CLI / Server
        |
        v
pdfrust-thumbnail API
        |
        +-- pdfium backend       (first product backend)
        |
        +-- rust backend         (incremental replacement)
        |
        +-- comparison harness   (renders both, diffs output)
```

The public API should be backend-neutral from day one. That prevents the
short-term PDFium bridge from becoming the permanent product API.

## Thumbnail API Sketch

Rust:

```rust
pub struct ThumbnailOptions {
    pub page_index: Option<u32>,
    pub max_edge: Option<u32>,
    pub background: Background,
    pub format: OutputFormat,
    pub timeout_ms: Option<u64>,
}

pub struct Thumbnail {
    pub width: u32,
    pub height: u32,
    pub format: OutputFormat,
    pub bytes: Vec<u8>,
}

pub trait ThumbnailBackend {
    fn render_thumbnail(&self, input: PdfInput<'_>, options: ThumbnailOptions) -> Result<Thumbnail>;
}
```

Default behavior:

- `page_index`: `0`.
- `max_edge`: `1024`.
- `format`: PNG for CLI artifacts, RGBA for differential tests.
- `timeout_ms`: `5000`.
- backend execution: serialized for PDFium.

Errors should be mapped into stable classes:

- password or encrypted PDF,
- malformed PDF,
- unsupported feature,
- timeout,
- internal error.

Node:

```ts
renderThumbnail(input, {
  pageIndex: 0,
  maxEdge: 1024,
  background: '#ffffff',
  format: 'png',
  timeoutMs: 5000
})
```

## Milestones

### Milestone 1: Cut-Down PDFium Probe

- Build PDFium from source with V8 and XFA disabled.
- Render page index `0` from a small fixture set.
- Measure binary size, cold start, first-page render time, thumbnail render
  time, and memory high-water mark.
- Confirm output quality for invoices, browser-generated PDFs, scanned PDFs,
  vector-heavy PDFs, and password-protected PDFs.
- Use a serialized PDFium backend for the first probe.
- Keep generated fixtures in Git and keep the curated real-world corpus outside
  Git.

Exit criteria:

- One CLI command can produce thumbnails from fixtures.
- Build flags and binary size are documented.
- The unsupported/error behavior is explicit.
- The probe does not promise npm distribution, prebuilt binaries, or bundled
  PDFium.

### Milestone 2: Backend-Neutral API

- Add Rust facade crate with `ThumbnailBackend`.
- Implement PDFium backend behind that trait.
- Return PNG or RGBA.
- Keep PDFium handles private.

Exit criteria:

- A Rust caller can generate a thumbnail without knowing PDFium exists.
- The fixture suite can exercise the facade with PDFium as the active backend.

Node-API remains a planned layer after Phase 0, not a milestone in the first
probe.

### Milestone 3: Differential Harness

- Store fixture metadata.
- Render with PDFium backend.
- Render with experimental Rust backend when available.
- Compare dimensions, error class, and pixels.
- Record allowed tolerances.

Exit criteria:

- Future Rust rendering work has an objective pass/fail loop.

### Milestone 4: Rust Parser And Simple Renderer

- Parse enough structure to load simple generated PDFs.
- Render basic vector pages and embedded JPEG images.
- Produce thumbnails for a deliberately narrow fixture subset.

Exit criteria:

- The Rust backend can pass a small nontrivial thumbnail fixture set.
- Unsupported files fail with typed errors, not panics.

## Planning Recommendation

Do the cut-down PDFium probe before any large Rust porting work. It will answer
the most important practical questions:

- How small can the real PDFium thumbnail build get?
- Which dependencies remain unavoidable?
- Is the single-threaded API constraint acceptable for the target workload?
- What error cases matter in the real documents?
- How much fidelity is actually needed for thumbnails?

If the PDFium bridge is already small and operationally acceptable, the Rust
port can proceed as a long-term replacement without blocking the product need.
If the PDFium bridge is too heavy, the Rust-native work should stay aggressively
thumbnail-only and avoid broader PDFium parity.

Phase 0 should not decide npm packaging, prebuilt binaries, Node-API details,
or whether PDFium is shipped as a product backend.
