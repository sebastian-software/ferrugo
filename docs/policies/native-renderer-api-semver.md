# Native Renderer API And Semver Policy

Status: accepted for 0156.
Date: 2026-06-26.

This policy defines the public Rust-native renderer boundary after normal
runtime behavior became PDFium-free. It separates consumer APIs from maintainer
tooling so renderer internals can keep changing without forcing downstream
application changes.

## Public Consumer Boundary

The 1.0 stable consumer boundary is:

- `ferrugo-thumbnail`: backend-neutral source, options, thumbnail, metadata,
  backend trait, and error taxonomy types.
- `ferrugo-native::NativeBackend`: native backend construction, render limits,
  memory diagnostics, single-page rendering through `ThumbnailBackend`,
  metadata inspection through `DocumentMetadataBackend`, first-page preview,
  and partial multi-page preview entry points.
- `ferrugo` default commands: native-only `render`, `render-auto`, and
  `render-native` behavior for smoke tests and operational automation.

The stable Rust consumer surface is intentionally narrow:

| Surface | 1.0 contract |
| --- | --- |
| `PdfSource`, `ThumbnailOptions`, `Thumbnail`, `PixelFormat`, `OutputFormat`, `Rgba` | Stable thumbnail input, output, and options. Existing fields and enum variants stay compatible through the 1.x line. |
| `ThumbnailBackend`, `DocumentMetadataBackend`, `TextExtractionBackend` | Stable backend-neutral traits for render, metadata, and bounded text extraction. |
| `DocumentMetadata` and nested metadata structs | Stable bounded metadata result shapes for consumer inspection. Existing fields stay compatible through the 1.x line. |
| `ThumbnailError`, `ThumbnailErrorClass`, `unsupported_feature_buckets`, `STABLE_UNSUPPORTED_FEATURE_BUCKETS` | Stable high-level error classes and stable unsupported-feature diagnostic buckets. |
| `NativeBackend`, `NativeRenderLimits`, `NativeMemoryDiagnostics`, first-page preview, partial preview, and document-session entry points | Stable native backend entry points and budget/profile data needed by server/runtime consumers. |

Consumer APIs must not expose PDFium handles, PDFium-specific error values, or
PDFium fallback state. PDFium remains an optional maintainer oracle behind the
`pdfium` feature, not part of the normal API contract.

## CLI Consumer Contract

The 1.0 stable CLI consumer contract is the native rendering path:

| Command | Contract |
| --- | --- |
| `ferrugo render` | Render one page with the Rust-native backend and write PNG output. |
| `ferrugo render-auto` | Alias for the native runtime path; no runtime PDFium fallback is attempted. |
| `ferrugo render-native` | Force the native backend explicitly for scripts that want the backend in the command name. |
| `ferrugo --version` / `ferrugo -V` | Print `ferrugo <version>`. |
| `ferrugo --help` / `ferrugo -h` | Print command usage. Help text may gain commands or options but must keep the stable native render options visible. |

Stable native render options are:

- positional input PDF path;
- `--output PATH` / `-o PATH`;
- `--page-index N`;
- `--max-edge N`;
- `--background #RRGGBB` or `#RRGGBBAA`;
- `--timeout SECONDS`;
- `--annotation-mode screen|print`;
- `--native-only` and `--no-pdfium-fallback` as compatibility no-ops because
  the native path is already PDFium-free.

CLI success exits with status 0 and writes the requested PNG. CLI failures exit
non-zero, write a human-readable error to stderr, and preserve the
`render error [<class>]` prefix for native render errors where `<class>` is one
of the stable `ThumbnailErrorClass::as_str()` values.

`--allow-pdfium-fallback` remains a stable rejection on native commands. It must
not silently re-enable runtime PDFium fallback.

## Maintainer And Internal Boundary

The following surfaces are not committed as stable application APIs:

- `ferrugo-content`, `ferrugo-object`, `ferrugo-render`, and `ferrugo-syntax`
  low-level parser, object, display-list, and raster internals.
- `ferrugo-pdfium` and PDFium-specific CLI commands such as `render-pdfium`,
  `render-isolated`, `compare-metadata`, `benchmark-pdfium`, and `visual-diff`.
- Maintainer CLI commands and reports such as `summarize-fallbacks`,
  `operator-coverage`, `trace-native`, `replay-operators`,
  `extract-corpus-metadata`, `producer-regression-report`,
  `classify-pdf20-usage`, `validate-local-corpus`, `benchmark-native`,
  `benchmark-batch-native`, `benchmark-repeat-native`, `benchmark-matrix`, and
  `visual-diff-poppler`.
- Exact visual-diff thresholds, fixture manifests, benchmark JSON shape, and
  conformance triage reports.
- Low-level renderer diagnostics beyond the stable unsupported-feature buckets
  exposed by `ferrugo-thumbnail`, including native trace, route summary,
  operator coverage, benchmark, cache, and visual-oracle JSON shapes.
- Public Rust types used mainly by maintainer tooling, including native trace,
  timing, operator coverage, raster route, cache summary, and benchmark helper
  structures, unless they are also listed in the public consumer boundary.

Internal crates can change between release slices as long as the public consumer
boundary above continues to build, test, and preserve documented behavior.

## Extensibility Decision

The 1.0 surface does not add `#[non_exhaustive]` to the stable consumer enums or
replace stable option/result structs with builders. That decision keeps the
existing Rust facade straightforward for the first stable server/runtime line.

The cost is explicit: adding enum variants, removing variants, changing stable
field names or types, or adding required fields to stable public structs is a
breaking change for the 1.x line. New optional convenience constructors,
builders, helper methods, and trait implementations may be added later when
existing literal construction, defaults, and trait calls keep compiling.

Maintainer-only public types can still change during the 0.x line. Before a 1.0
release, any maintainer type that should become consumer-stable must be moved
into the public consumer boundary table above and covered by examples or tests.

## Semver Rules

Until the PDFium-free 1.0 release, each public crate stays on the `0.x` train:

- Patch releases must not intentionally break public Rust signatures,
  `ThumbnailErrorClass::as_str()` values, or default native runtime behavior.
- Minor releases may include planned public API cleanup only when the release slice
  includes migration notes and the package dry-run passes.
- Stable consumer structs with public fields are treated as
  literal-construction compatible. Adding, removing, or renaming a field is a
  breaking change for the 1.x line.
- Stable consumer enums are exhaustive. Adding variants is a breaking change for
  consumers that match exhaustively.
- New inherent methods, trait implementations, and new optional CLI commands
  are non-breaking when existing behavior remains unchanged.

After 1.0, the project follows standard SemVer:

- Major releases may break public Rust signatures or stable string values.
- Minor releases may add non-breaking APIs and new documented diagnostics.
- Patch releases are bug fixes, performance fixes, and documentation updates
  that preserve the public contract.

## Error And Diagnostic Compatibility

`ThumbnailErrorClass::as_str()` values are stable metadata-safe class names:
`encrypted`, `malformed`, `unsupported`, `timeout`, and `internal`. These values
are safe for logs, CLI automation, and baseline metadata.

`ThumbnailError` variants are stable high-level failure classes. The
`UnsupportedFeature(&'static str)` bucket gives consumers and maintainers a more
precise native boundary while preserving the public `unsupported` class through
`ThumbnailError::class()`. The bucket constants in
`ferrugo_thumbnail::unsupported_feature_buckets` and the
`STABLE_UNSUPPORTED_FEATURE_BUCKETS` list are stable diagnostic strings.
Consumers should branch on `class()` for coarse fallback behavior and use
`unsupported_feature_bucket()` only for feature-specific telemetry, support
messages, or explicit alternate-renderer routing.

Internal `Internal(String)` messages are not stable. They may change to improve
debuggability and must not be used as control-flow keys.

## Rendering Options And Defaults

`ThumbnailOptions::default()` remains the stable thumbnail contract:

- page index `0`
- maximum edge `1024`
- opaque white background
- raw RGBA output
- five second timeout

Changing these defaults is a breaking behavior change. New options should be
introduced through new fields only after the struct extensibility question is
resolved, or through new builder/newtype APIs that do not invalidate existing
literal construction.

`NativeRenderLimits::default()` and `NativeBackend::low_memory()` are documented
profiles, not exact performance promises. Their fields are public today, so
field-shape changes are breaking. Numeric default values may be tightened only
when the supported corpus, benchmark budget, and low-memory gates remain green.

## Migration From PDFium-Backed APIs

Consumers should migrate to native-only runtime behavior by:

1. Depending on `ferrugo-thumbnail` plus `ferrugo-native` for library usage.
2. Using `NativeBackend::new()` or `NativeBackend::low_memory()` and the
   backend-neutral `ThumbnailBackend` / `DocumentMetadataBackend` traits.
3. Treating `ThumbnailError::class()` as the stable failure key.
4. Using `render`, `render-auto`, or `render-native` for CLI automation without
   enabling the `pdfium` feature.
5. Reserving PDFium-enabled commands for maintainer oracle checks and visual
   diffs only.

Applications that previously depended on PDFium fallback should now handle
`unsupported` as a typed native outcome. Feature-specific handling can use the
stable unsupported bucket constants instead of parsing display strings.
