# Native Renderer API And Semver Policy

Status: accepted for 0156.
Date: 2026-06-26.

This policy defines the public Rust-native renderer boundary after normal
runtime behavior became PDFium-free. It separates consumer APIs from maintainer
tooling so renderer internals can keep changing without forcing downstream
application changes.

## Public Consumer Boundary

The stable consumer boundary is:

- `pdfrust-thumbnail`: backend-neutral source, options, thumbnail, metadata,
  backend trait, and error taxonomy types.
- `pdfrust-native::NativeBackend`: native backend construction, render limits,
  memory diagnostics, single-page rendering through `ThumbnailBackend`,
  metadata inspection through `DocumentMetadataBackend`, first-page preview,
  and partial multi-page preview entry points.
- `pdfrust-cli` default commands: native-only `render`, `render-auto`, and
  `render-native` behavior for smoke tests and operational automation.

Consumer APIs must not expose PDFium handles, PDFium-specific error values, or
PDFium fallback state. PDFium remains an optional maintainer oracle behind the
`pdfium` feature, not part of the normal API contract.

## Maintainer And Internal Boundary

The following surfaces are not committed as stable application APIs:

- `pdfrust-content`, `pdfrust-object`, `pdfrust-render`, and `pdfrust-syntax`
  low-level parser, object, display-list, and raster internals.
- `pdfrust-pdfium` and PDFium-specific CLI commands such as `render-pdfium`,
  `render-isolated`, `compare-metadata`, `benchmark-pdfium`, and `visual-diff`.
- Exact visual-diff thresholds, fixture manifests, benchmark JSON shape, and
  conformance triage reports.
- Low-level renderer diagnostics beyond the stable unsupported-feature buckets
  exposed by `pdfrust-thumbnail`.

Internal crates can change between milestones as long as the public consumer
boundary above continues to build, test, and preserve documented behavior.

## Semver Rules

Until the PDFium-free 1.0 release, each public crate stays on the `0.x` train:

- Patch releases must not intentionally break public Rust signatures,
  `ThumbnailErrorClass::as_str()` values, or default native runtime behavior.
- Minor releases may include planned public API cleanup only when the milestone
  includes migration notes and the package dry-run passes.
- Public structs with public fields are treated as literal-construction
  compatible. Adding, removing, or renaming a field is a breaking change unless
  the type is first explicitly marked and documented as extensible.
- Public enums are exhaustive today. Adding variants is a breaking change for
  consumers that match exhaustively unless the enum is first explicitly marked
  and documented as extensible.
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
`pdfrust_thumbnail::unsupported_feature_buckets` and the
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

1. Depending on `pdfrust-thumbnail` plus `pdfrust-native` for library usage.
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
