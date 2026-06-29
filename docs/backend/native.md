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

## API And Semver Policy

The native consumer API is the `pdfrust-thumbnail` facade plus
`pdfrust-native::NativeBackend`. PDFium types, handles, and fallback state are
not part of this boundary. Public error classes, default thumbnail options, and
native preview entry points are covered by the API policy in
`docs/policies/native-renderer-api-semver.md`.

Renderer internals and maintainer diagnostics can continue to evolve behind the
stable facade. Planned public cleanup before 1.0 is tracked in
`docs/backlogs/native-renderer-api-cleanup-backlog.md`.

## Untrusted Input And Fuzz Boundary

The native backend treats PDF bytes as untrusted input. Parser, font, image, and
raster paths use explicit recursion, byte, pixel, cache, and decoded-sample
budgets; exceeding those limits returns typed public errors rather than
attempting unbounded repair or allocation.

The 0139 refresh adds a minimized huge-image-dimensions adversarial PDF and
checks declared image sample sizes before image stream decoding. Current fuzz
smoke targets cover primitive parsing, xref setup, stream decoding, content
tokenization, and native render setup. See `docs/fuzzing.md` and
`docs/reports/native-renderer-security-fuzz-refresh-2026-06-26.md`.

## GA Gate Status

The 2026-06-26 GA2 gate keeps the native renderer in a conditional state:
core supported-family technical execution is PDFium-free, but broad visual GA
is not declared yet. The native-only gate renders all current `browser-print`,
`office-export`, and `form` fixtures without fallback or errors. The full
typical corpus is 146/155 native-rendered with 8 typed unsupported boundaries
and 1 encrypted error.

The PDFium visual oracle still reports material blockers in form appearance
parity, text/font rendering, rendering-core details, image/color parity, and
page geometry. PDFium should remain out of normal native-only runtime paths, but
stay available as explicit maintainer comparison tooling. See
`docs/reports/native-renderer-ga2-coverage-2026-06-26.md` for the measured
decision.

The 0143 conformance triage loop keeps the same runtime decision and routes the
visual blockers into subsystem-owned follow-up slices. The current full-corpus
visual oracle reports 91 blockers, 23 accepted drift rows, 8 native unsupported
rows, and 1 encrypted both-error row. The highest-priority owner areas are
`text-fonts`, `rendering-core`, `annotations-forms`, `images-color`, and
`page-geometry`. See
`docs/reports/native-renderer-conformance-triage-2026-06-26.md` and
`docs/backlogs/native-renderer-conformance-backlog.md`.

The 0144 operator audit adds a native content-stream operator coverage scan.
The current generated corpus scan covers 154/155 fixtures, with the remaining
row being the expected encrypted placeholder. It records 5,565 operators:
5,472 implemented, 85 partial, 0 unsupported, and 8 intentionally ignored
marked-content operators. Partial operator work is concentrated in `gs`, `W`,
color-space operators, and `sh`. See
`docs/reports/renderer-operator-coverage-audit-2026-06-26.md`.

## Page Artifact Cache Policy

The native renderer default policy is `isolated-render`: each thumbnail render
owns its decoded page resources and pass-local caches, and no document-derived
artifact is persisted to disk by default. Longer-lived page reuse remains
caller-owned until the backend grows an explicit document-session cache with
bounded memory accounting and tenant lifetime boundaries.

Reusable page artifacts must be keyed by `NativePageCacheKey`, which includes a
caller-provided document identity, page index, max edge, background color,
native renderer version, native profile, annotation mode, and AcroForm
appearance mode. The CLI repeated-render benchmark uses a streaming content
hash as the document identity for fixture evidence; host applications may
instead provide a tenant-scoped document version id or a strong content hash.

The native backend renders AcroForm document state by default. Existing widget
appearance streams and `/AS` appearance-state selection remain authoritative.
Explicit viewer-side form mutation preview requests are rejected with
`unsupported` bucket `form.appearance-mutation` rather than updating field
values, appearance dictionaries, or flattened page bytes during thumbnail
rendering.

The current repeated-render gate does not show enough improvement to justify
shared persistent page artifacts as a default: the 0134 benchmark rendered four
fixtures three times each with 0 fallbacks and 0 budget failures, while repeated
render means stayed close to first-render timings. This rejects a global or
on-disk page cache for now and keeps future cache experiments behind explicit
policy and key boundaries. See
`docs/reports/page-cache-reuse-policy-2026-06-25.md`.

The bounded multi-page renderer shares one immutable parsed document and page
tree across the pages requested in a single render call. This removes repeated
object-table parsing for long-document preview batches while keeping decoded
fonts, images, raster surfaces, and page-local caches owned by each page render.
Single-page page-zero requests can still use the linearized first-page loader;
multi-page requests intentionally require the full classic loader so later pages
never see an incomplete first-page object table. See
`docs/reports/shared-resource-cache-2026-06-29.md`.

## Incremental Preview Memory

`NativeBackend::render_first_page_preview` now reports
`FirstPagePreviewMemory` alongside the rendered thumbnail and load mode. The
metrics expose total input bytes, parsed object count, parsed object byte span,
the declared linearized first-page section size, and whether the loader kept to
that first-page section.

For valid linearized local inputs, page-zero preview uses the bounded
first-page object loader and avoids retaining parsed objects past the
first-page section. For malformed linearization hints, non-linearized files,
remote transports, and pages other than page zero, correctness wins: the native
backend falls back to full-file availability rather than guessing a partial
object graph. See
`docs/reports/incremental-streaming-memory-budget-2026-06-29.md`.

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

## Font Subset Regression Coverage

The font-subset regression corpus covers reduced TrueType, CFF, Type0 CID,
Type3, and missing-font subset cases from common office and report producer
patterns. The fixtures live in `fixtures/font-subset-manifest.tsv` and are also
listed in the main corpus manifest under `office-export`.

| Family | Fixture | Coverage focus |
| --- | --- | --- |
| `truetype-subset` | `subset-truetype-widths.pdf` | Subset-prefixed TrueType with explicit `/Widths` and `FontFile2`. |
| `cff-subset` | `subset-cff-tounicode.pdf` | Subset-prefixed CFF `FontFile3` with explicit ToUnicode mapping. |
| `cid-subset` | `subset-cid-widths.pdf` | Type0 CID font with descendant width overrides and ToUnicode. |
| `type3-subset` | `subset-type3-repeated-charprocs.pdf` | Repeated Type3 CharProc reuse and Type3 width advancement. |
| `missing-font-subset` | `subset-missing-font.pdf` | Subset-prefixed missing font routed through deterministic fallback. |

The 0136 gate renders all five fixtures through the Rust-native backend with
zero fallbacks, zero errors, and zero benchmark budget failures. The current
run keeps mean render time below 1.2 ms per family at `max_edge = 160`. See
`docs/reports/font-subset-regression-2026-06-26.md`.

## Image Decode And Sampling Optimization

The native image path keeps decoded source samples as the only full-image
sample buffer for supported image XObjects. Flate PNG predictor reversal now
mutates the decoded buffer in place and truncates it to the final sample length,
avoiding a second full decoded-sample allocation for predictor images.

During image painting, the rasterizer samples only target thumbnail pixels. A
per-draw single-entry `ImageSampleCache` reuses the last converted RGBA sample
when multiple target pixels map to the same source pixel. This reduces repeated
CMYK, Indexed, Gray, stencil-mask, and soft-mask conversion work during common
thumbnail scaling without retaining a full RGBA intermediate image.

The 0137 image-heavy gate renders the supported mobile scan, photo scan,
OCR-over-image, mixed compression, DCT, and predictor fixtures with zero native
fallbacks, zero errors, and zero benchmark budget failures. PDFium visual
comparison still records the known scan resampling parity blockers, so image
resampling fidelity remains a separate backlog item. See
`docs/reports/image-downsampling-color-optimization-2026-06-26.md`.

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
the normal image and path paths. The raster path does not run OCR; it preserves
the visual boundary and avoids unnecessary raster work. See
`docs/reports/ocr-invisible-text-layer-2026-06-25.md`.

## Optional Content And Layer State

The native renderer applies catalog default optional-content state when
flattening thumbnails. `/OCG` resources referenced from marked content respect
`/OCProperties /D /BaseState`, `/ON`, and `/OFF`; nested marked-content scopes
are hidden when any enclosing optional-content group is hidden.

`DocumentMetadata.optional_content` exposes bounded layer policy signals:
catalog OCG count, default base state, default-on/default-off counts, and flags
for unsupported usage applications, membership dictionaries, and direct OCG
dictionaries. Rendering still rejects `/D /AS` usage application arrays and
`/OCMD` policies with `graphics.optional-content`, but metadata inspection
classifies those boundaries so consumers can route fallback or flattening
decisions deterministically. See
`docs/policies/optional-content.md` and
`docs/reports/optional-content-ui-state-2026-06-29.md`.

## Text Extraction

The native backend exposes a bounded `TextExtractionBackend` implementation for
one-page search use cases. It returns text runs in content-stream order with
decoded Unicode, per-glyph page-space origins, font size, and a `visible` flag
derived from the PDF text rendering mode.

Invisible OCR layers are searchable through this API while remaining invisible
to raster output. Extraction has explicit run and glyph limits and reports
`truncated = true` when those limits are reached. It does not perform semantic
document understanding, producer repair, exact tagged reading-order recovery,
selection highlighting, or OCR. See
`docs/reports/native-text-extraction-search-boundary-2026-06-29.md`.

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

## Incremental Preview Boundary

`NativeBackend::render_first_page_preview` is the explicit page-zero preview
entry point. It forces `page_index = 0`, renders through the native backend, and
reports whether the linearized first-page loader was usable or whether the
render fell back to full-document loading.

`NativeBackend::render_preview_pages_partial` exposes the partial preview
boundary for multi-page callers. It preserves page-level success and error
status, honors cooperative cancellation before scheduling further page batches,
and applies the render limits of the backend instance. This is still a local
byte-source preview API; remote range streaming and viewer UI behavior are
outside the current boundary. See
`docs/reports/incremental-preview-boundary-2026-06-26.md`.

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

The 0191 visual-review slice adds business-style Separation and DeviceN samples
plus a CMYK-alternate tint swatch under
`fixtures/spot-color-visual-review-manifest.tsv`. Those samples are regression
coverage for understandable thumbnail approximations, not color-managed
proofing. See
`docs/reports/devicen-spot-color-visual-review-2026-06-29.md`.

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

## Transparency And Blend Conformance

The transparency conformance manifest is
`fixtures/transparency-conformance-manifest.tsv`. It separates supported native
coverage from typed unsupported boundaries:

| Area | Native status |
| --- | --- |
| ExtGState fill/stroke alpha | Supported with known stroke-edge visual blocker. |
| Isolated transparency groups | Supported with exact or accepted-drift PDFium comparison. |
| Knockout group metadata | Parsed; current thumbnail behavior follows PDFium comparison output. |
| `Normal`, `Multiply`, `Screen` blend modes | Supported. |
| Blend-mode arrays | Supported when a later entry is one of the supported blend modes. |
| Image soft masks | Supported for DeviceGray image masks matching image dimensions. |
| ExtGState `/SMask /None` | Accepted. |
| ExtGState luminosity soft masks | Typed `graphics.transparency` fallback. |
| Advanced blend modes such as `Overlay` | Typed `graphics.transparency` fallback. |

The 0138 gate renders seven supported conformance rows natively with zero
fallbacks and classifies two unsupported rows under `graphics.transparency`.
See `docs/reports/transparency-blend-conformance-2026-06-26.md`.

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

## Print Imposition Thumbnails

Static print-imposed PDFs such as booklet spreads and n-up sheets render
natively as thumbnails when their content is expressed as ordinary page
geometry, vector marks, text, and page boxes. The native thumbnail policy uses
CropBox as the visible page boundary when present and MediaBox otherwise;
BleedBox and TrimBox remain metadata/context unless a future API adds explicit
page-box selection.

The renderer does not build imposed sheets, interpret printer marks
semantically, perform trapping, run preflight validation, or provide
color-managed proofing. See
`docs/reports/print-imposition-booklet-coverage-2026-06-29.md`.

## PDF/A And Archival Metadata

The native metadata path exposes archival profile signals without validating
compliance. `DocumentMetadata.archival` reports bounded XMP `pdfaid:part` and
`pdfaid:conformance` markers, catalog OutputIntent presence, and the stable
fact that conformance validation was not performed.

PDF/A markers do not change rasterization by themselves. Archive records render
natively when their page content uses supported graphics, image, text, font,
and page-geometry features. Embedded files remain inert metadata context and
OutputIntents remain thumbnail context rather than color-managed proofing. See
`docs/policies/pdfa-archival-boundary.md` and
`docs/reports/pdfa-archival-boundary-2026-06-29.md`.

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

## Annotation Preview Modes

Annotation rendering supports screen-style and print-preview-style visibility
through `ThumbnailOptions.annotation_mode`. Screen mode renders static
annotation appearances unless the annotation flags mark them hidden, invisible,
or no-view. Print mode renders only annotations whose `/F` flags include
`Print`; hidden and invisible annotations remain suppressed, while `NoView`
does not suppress print output.

Existing normal appearance streams remain authoritative. Missing appearances
are synthesized only for the bounded markup/widget subset documented in
`docs/policies/annotation-fallbacks.md`. Appearance-free FreeText annotations
return typed `annotation.appearance` unsupported rather than guessing text
layout. See
`docs/reports/annotation-print-preview-fidelity-2026-06-29.md`.

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

The CLI batch benchmark can fan out multiple page jobs per input with
`--pages-per-input`. When a manifest is supplied, the benchmark bounds that
fanout by each fixture's declared page count, preserving deterministic
repetition, input, and page-index ordering while keeping worker and
in-flight-pixel limits explicit.

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

`render` and `render-auto` use the Rust-native backend only. If native returns
the public `unsupported` class, the command returns a stable render error that
names the unsupported bucket; it does not retry through PDFium. The selected
backend is printed as a render diagnostic for successful native renders.

Use `render-native` when scripts must make the native-only choice explicit.
`--native-only` and `--no-pdfium-fallback` are accepted for compatibility, but
the normal render paths are already native-only. `--allow-pdfium-fallback` is
rejected because runtime PDFium fallback has been removed.

Use `render-pdfium`, `render-isolated`, `benchmark-pdfium`, `compare-metadata`,
or `visual-diff` only in a CLI build compiled with `--features pdfium`; those
commands are maintainer comparison tooling, not runtime fallback paths.

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

Add `--diagnostics-dir <path>` to emit one safe JSON diagnostic bundle for each
fixture that returns fallback-required or error. Bundles include render options,
manifest metadata, safe page count/page sizes, stage timings, typed error
class/category, a coarse stage hint, and native memory diagnostics. They do not
include PDF bytes, rendered pixels, or document-info fields such as title or
author by default. Review the path and manifest notes before sharing a bundle
outside the document trust boundary. See
`docs/policies/telemetry-diagnostics-privacy.md` and
`docs/reports/renderer-diagnostics-bundle-2026-06-25.md`.

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
