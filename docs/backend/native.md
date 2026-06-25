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
approximation and does not expose separations, unbounded function sampling, or
overprint simulation. See
`docs/reports/spot-color-approximation-2026-06-25.md`.

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
