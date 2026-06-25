# Rust-Native Backend

Status: supported for generated fixture coverage.
Date: 2026-06-24.

The Rust-native backend lives in `crates/pdfrust-native` and implements the same
`pdfrust-thumbnail` facade traits as the PDFium backend:

- `ThumbnailBackend` for single-page RGBA thumbnail rendering.
- `DocumentMetadataBackend` for page-count and page-size inspection.

Callers can switch between `PdfiumBackend` and `NativeBackend` without changing
the facade input or output types. The native backend returns raw RGBA thumbnail
buffers through `Thumbnail::rgba`; CLI PNG encoding remains owned by
`pdfrust-cli`, matching the PDFium backend path.

## Supported Contract

- `page_index` selects the zero-based page or returns `unsupported` when the
  page is unavailable.
- `max_edge` uses the same scale-down policy as PDFium: the largest page edge is
  clamped to the requested maximum, preserving aspect ratio.
- `background` initializes the page raster before paths, images, text, and
  appearances are painted.
- Encrypted documents return the public `encrypted` class.
- Malformed parser/object structures return the public `malformed` class.
- Unsupported native features and budget exhaustion return the public
  `unsupported` class.

## GA Gate Status

The 2026-06-25 GA gate keeps the native renderer in a conditional state:
supported-family technical execution is PDFium-free, but broad visual GA is not
declared yet. The supported-family native-only gate renders `browser-print`,
`office-export`, and `form` fixtures without fallback, while the PDFium visual
baseline still reports fidelity blockers in form synthesis, text/font rendering,
and page geometry. See
`docs/reports/native-renderer-ga-gate-2026-06-25.md` for the measured decision.

## Font Fallback Policy

Missing and substituted fonts use a deterministic built-in fallback policy
rather than host operating-system font lookup. The current policy classifies
font names after subset-prefix stripping and maps them to `Sans`, `Serif`,
`Monospace`, or `Symbol` fallback faces. Type 3 fonts remain routed through
their CharProc paths instead of the built-in text fallback.

Fallback resolutions are cached with a bounded default limit of 128 entries and
the cache key stores only classified fallback metadata, not document-specific
font names. The limit is exposed as `max_font_fallback_cache_entries` through
native memory diagnostics. See
`docs/reports/font-fallback-policy-2026-06-25.md` for the current evidence and
PDFium comparison results.

## Font Program Safety

Glyph outline extraction is bounded by path segment, cache, charstring stack,
and charstring subroutine-depth limits. CFF programs use the bounded
`ttf-parser` path. Type1 FontFile programs have a small native charstring subset
interpreter for common move, line, curve, close, width, and divide operators.
Malformed charstrings, unsupported operators, stack overflow, and subroutine
attempts return typed glyph-outline errors mapped to `text.glyph-outline`.

The current text rasterizer still uses built-in bitmap fallback for visible
non-Type3 text, so CFF/Type1 native rendering is no-fallback but not yet visual
parity with PDFium. See
`docs/reports/cff-type1-charstring-hardening-2026-06-25.md`.

## Text Layout Fallback Policy

Decoded PDF glyph metadata now records a native `TextLayoutStatus`. The native
renderer classifies simple glyphs, ToUnicode ligature expansions, combining-mark
sequences, pre-positioned shaped scripts, and typed unsupported complex-script
fallbacks. This makes shaped-text behavior visible to diagnostics instead of
silently collapsing all text into the same ASCII fallback path.

Fallback rasterization expands one PDF source glyph into all mapped Unicode
scalars, so common ligature mappings such as `fi` are rendered as visible native
fallback text. Combining marks are positioned over the previous base glyph by a
small deterministic mark fallback. Repeated fallback rasterization reuses
scratch capacity for expanded text atoms.

This is still not full OpenType GSUB/GPOS table shaping. The current milestone
handles PDF-exported shaped output and records typed unsupported reasons for
cases outside that subset. See
`docs/reports/opentype-layout-feature-coverage-2026-06-25.md`.

## OCR And Invisible Text Layers

The text display-list path preserves invisible text runs, including OCR layers
that use PDF text rendering mode `3`, so future metadata and search extraction
can still inspect decoded text separately from visual output. Rasterization uses
`TextRenderingMode::paints_pixels()` to skip glyph bitmap lookup, scratch buffer
expansion, and pixel compositing for modes that cannot paint pixels.

This keeps searchable scan-style PDFs visually faithful: hidden OCR text does
not appear in thumbnails, while the visible scan artwork still renders through
the normal image and path paths. The current slice does not expose a text search
API or run OCR; it only preserves the visual boundary and avoids unnecessary
raster work. See
`docs/reports/ocr-invisible-text-layer-2026-06-25.md`.

## Tagged PDF Accessibility Metadata

Native metadata inspection now exposes a bounded accessibility metadata block
separate from visual rendering. The current signals include catalog `/Lang`,
`/MarkInfo /Marked`, RoleMap presence, counted structure element roles, and
marked-content references reached through the structure tree.

Structure traversal is capped at 4096 reached values and tracks visited
indirect objects to avoid cycles. Malformed optional structure-tree content
returns the stable `malformed` metadata error class instead of affecting page
rasterization. Tagged and malformed structure fixtures both continue to render
through the native backend without requiring accessibility metadata success.
See `docs/reports/tagged-pdf-accessibility-metadata-2026-06-25.md`.

## CMap And Identity Text Decoding

ToUnicode CMaps support explicit `begincodespacerange`, `beginbfchar`, and
`beginbfrange` sections with bounded decoded bytes and entry counts. Text lookup
uses longest matching source codes and respects parsed code-space ranges.

Type0 fonts that use `/Encoding /Identity-H` or `/Encoding /Identity-V` without
a ToUnicode stream receive a bounded two-byte identity fallback map. This keeps
common synthetic and subset CID fixtures native-renderable rather than falling
back to PDFium. ToUnicode CMaps may also use `/Identity-H usecmap` or
`/Identity-V usecmap` as an explicit base. Other named `usecmap` references
remain unsupported until named CMap resource lookup and cycle detection exist.
See `docs/reports/cmap-identity-coverage-2026-06-25.md`.

## Spot Color Approximation

Page `/ColorSpace` resources now support common `/Separation` and `/DeviceN`
spot-color spaces for vector fill and stroke content. The native renderer
evaluates bounded Type 2 tint transforms and converts DeviceGray, DeviceRGB,
and DeviceCMYK alternate spaces into RGB thumbnail output. Captured colors use
`DeviceColor::Spot` so callers and reports can distinguish approximated spot
color from direct device RGB/Gray input.

This is not a press-proofing implementation. Native output is an RGB thumbnail
approximation and does not expose separations or unbounded function sampling.
Overprint graphics-state flags are accepted as a separate thumbnail
approximation path described below. See
`docs/reports/spot-color-approximation-2026-06-25.md`.

## ICCBased Image Color Spaces

Image XObjects now accept `/ICCBased` color spaces with bounded decoded profile
streams. Supported channel counts map to the existing native image sample paths:
one component to DeviceGray, three components to DeviceRGB, and four components
to DeviceCMYK. Unsupported channel counts remain typed image color-space
failures.

Validated ICC transform metadata is reusable through a caller-owned
`IccTransformCache`. The cache is keyed by stable decoded-profile identity and
records hits, misses, evictions, and the largest validated transform workspace.
Default limits are exposed through native memory diagnostics:

- ICC profile bytes: 1 MiB.
- ICC transform workspace bytes: 64 KiB.
- ICC transform cache entries: 32.

This is not full color management. RGB and Gray ICCBased fixtures currently
match PDFium exactly, while CMYK ICCBased images render natively through the
DeviceCMYK thumbnail approximation and remain a known visual-parity gap. See
`docs/reports/icc-cache-transform-2026-06-25.md`.

## Tiling Pattern Color Spaces

The native vector renderer supports common colored and uncolored tiling
patterns. Colored patterns use paint from the pattern stream. Uncolored
patterns selected through `[/Pattern <base-space>]` use caller-supplied
DeviceGray, DeviceRGB, or DeviceCMYK operands from the `scn` fill-color
operator.

Pattern cell samples are cached within one rasterization pass. Cache keys
include the pattern resource name, paint mode, caller color for uncolored
patterns, and quantized transform scale. The default
`PathRasterOptions::max_pattern_cell_cache_entries` is 32 entries; setting it to
0 disables retained pattern cache entries. Pattern tile expansion remains
bounded by `PathRasterOptions::max_pattern_tiles`.

The generated colored and uncolored tiling pattern fixtures render natively and
match PDFium exactly in the 0107 comparison run. See
`docs/reports/tiling-pattern-color-spaces-2026-06-25.md`.

## Mesh Shading Tessellation

The native shading path resolves stream-backed `/Shading` resources and supports
a bounded subset of Type 4 free-form Gouraud triangle meshes. The current
implementation accepts `/DeviceGray` and `/DeviceRGB` meshes with 8-bit
coordinates, 8-bit color components, and 2- or 8-bit flags. The first slice
decodes explicit flag-0 triangle records; connected mesh continuation flags and
other mesh shading types remain typed unsupported boundaries.

Mesh streams are decoded with explicit renderer budgets:

- Decoded mesh stream bytes: 1 MiB.
- Decoded mesh triangles per shading: 8192.

Budget exhaustion is reported as `renderer.memory-budget` through the native
backend. This keeps adversarial or extremely dense mesh streams from scaling
work unboundedly with source complexity. The generated Type 4 fixture renders
natively and compares against PDFium as accepted low-amplitude drift in the 0108
run. See `docs/reports/mesh-shading-tessellation-2026-06-25.md`.

## Transparency Group Alpha

Transparency groups render into bounded intermediate rasters before being
composited back to the page. The group path preserves alpha on transparent
intermediate surfaces, applies the caller graphics-state alpha when the group is
painted, and uses the caller blend mode for final group compositing.

The group rasterizer now executes the full display list inside the intermediate
surface, not only path items. Intermediate size remains clipped to the
transformed group bounds and bounded by
`PathRasterOptions::max_transparency_group_pixels`.

`/K true` group metadata is parsed and covered by a generated fixture. The local
PDFium oracle renders the fixture overlap as normal semi-transparent group
composition, so the native renderer follows that comparison behavior for now
instead of introducing a divergent hard-knockout interpretation. See
`docs/reports/transparency-group-alpha-2026-06-25.md`.

## Overprint Approximation

ExtGState `/OP`, `/op`, and `/OPM` entries are parsed and validated by the
native renderer. Enabled stroking and nonstroking overprint no longer force a
PDFium fallback for thumbnail output. Instead, the flags are preserved on the
graphics state attached to display-list items, and the current RGB/spot-color
approximation is painted normally.

This is intentionally an RGB thumbnail approximation, not a press-proof
overprint simulator. The current path keeps common print-oriented documents
visible and diagnosable while leaving device-separation compositing, full CMYK
knockout behavior, and prepress conformance for later print-production
milestones. See `docs/reports/overprint-simulation-2026-06-25.md`.

## Digital Signature Boundary

Visible signature widgets use the same static AcroForm appearance rendering as
other widget annotations. Document metadata exposes presence-only signature
signals for AcroForm signature fields and `/ByteRange` dictionaries so callers
can distinguish signed-looking documents from unsigned forms without implying
cryptographic validation.

The native renderer does not validate certificate chains, hash signed byte
ranges, parse PKCS#7/CMS contents, or report legal signature status. See
`docs/reports/signature-boundary-2026-06-25.md`.

## Embedded Files And Portfolio Visibility

Embedded files stay inert in the native thumbnail path. Catalog
`/Names /EmbeddedFiles`, portfolio `/Collection`, and page
`/Subtype /FileAttachment` annotation presence are exposed through document
metadata so callers can classify these documents without opening attachments.

Attachment annotations render only through the existing annotation appearance
path when a normal appearance stream is present. The renderer does not extract,
open, execute, or preview embedded payloads, and it does not implement a
portfolio browser. The metadata scan remains bounded and reports presence
signals only. See `docs/reports/embedded-files-portfolio-2026-06-25.md`.

## Linearized First-Page Loading

For classic-xref PDFs that declare a valid linearization dictionary, page-zero
thumbnail rendering first attempts a bounded first-page load. The object layer
parses `/L`, `/E`, `/O`, `/N`, `/H`, and `/T` metadata, exposes loader metrics,
and loads only indirect objects whose xref offsets fall inside the declared
first-page section. If the linearization dictionary is absent, malformed,
incomplete, or insufficient for page-zero rendering, the native backend falls
back to the existing full classic loader.

This is not network range fetching. The current slice keeps the in-memory input
model and uses linearization metadata to reduce parsed object graph size for
the first page when the hints are valid. See
`docs/reports/linearized-first-page-loading-2026-06-25.md`.

## Multi-Page Scheduler And Cancellation

The native backend exposes a bounded multi-page scheduler for consumers that
need several thumbnails from the same document. `render_pages_parallel` keeps
the strict all-success behavior and preserves requested page order.
`render_pages_parallel_partial` returns page-level outcomes so callers can keep
successful pages when later requested pages fail.

Scheduling is bounded by `ParallelRenderOptions::max_workers` and
`max_in_flight_pixels`. `RenderCancellation` is a cooperative token checked
before worker batches are scheduled; already-started page jobs are allowed to
finish and release their temporary buffers normally. The scheduler does not
require an async runtime. See
`docs/reports/multi-page-scheduler-cancellation-2026-06-25.md`.

## Fallback Policy

PDFium remains the oracle and explicit fallback until the visual GA gate says
the native backend covers enough typical documents. Product code should use
native only where the support matrix marks the document class as rendered and
the caller accepts the documented fidelity level, or should explicitly retry
through PDFium when native returns `unsupported`.

Do not retry native `encrypted` or `malformed` errors through a silent repair
path. Encrypted documents need explicit password/security policy, and malformed
documents should stay diagnosable.

## Local Checks

Use native directly:

```sh
cargo run -p pdfrust-cli -- render-native fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page-native.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

Use native-first automatic rendering for supported categories:

```sh
cargo run -p pdfrust-cli -- render fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page-auto.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

`render` and `render-auto` try the Rust-native backend first. If native returns
the public `unsupported` class, the default behavior is a stable render error
that names the fallback bucket and asks for `--allow-pdfium-fallback` when the
caller wants PDFium retry behavior. In a PDFium-enabled CLI build,
`--allow-pdfium-fallback` retries through PDFium using
`PDFRUST_PDFIUM_LIBRARY`. `encrypted`, `malformed`, and `internal` failures are
not silently retried. The selected backend is printed as a render diagnostic.
Fallback diagnostics include `fallback_reason=<bucket>` and
`fallback_category=<bucket>` so corpus runs can count the remaining PDFium
surface.

Use `--allow-pdfium-fallback` with `render`/`render-auto` to permit explicit
PDFium retry. Use `--native-only` or `--no-pdfium-fallback` to force denial
after environment-driven fallback has been enabled. Use
`--deny-fallback-reason <bucket>` for targeted experiments, or set
`PDFRUST_ALLOW_PDFIUM_FALLBACK=1`, `PDFRUST_NATIVE_ONLY=1`, and
`PDFRUST_DENY_FALLBACK_REASONS=bucket.one,bucket.two` for environment-driven
runs.

Use `render-native` to force native without fallback. Use `render-pdfium` or
`render-isolated` to force PDFium in a CLI build compiled with
`--features pdfium`.

Summarize a local corpus without rendering PDFium output:

```sh
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/pdfrust-thumbnails/fallback-summary.json
```

The summary counts `native_rendered`, `fallback_required`,
`fallback_categories`, non-fallback `errors`, and per-family pass rates when a
manifest is provided. Add `--fail-on-fallback` for CI subsets that must stay
native-only. Add one or more `--include-family <family>` arguments with a
manifest to run a supported-category native-only gate:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family office-export \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160
```

Extract committed fixture metadata with page sizes and manifest tags:

```sh
cargo run -p pdfrust-cli -- extract-corpus-metadata fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --output target/pdfrust-thumbnails/corpus-metadata.json
```

Compare metadata with PDFium when the local PDFium environment is available:

```sh
cargo run -p pdfrust-cli --features pdfium -- compare-metadata fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page-metadata-comparison.json
```

The comparison JSON includes `rust_native_memory`, which records the default
native memory budget snapshot used for the local run.
