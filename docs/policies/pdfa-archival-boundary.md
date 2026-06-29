# PDF/A And Archival Boundary

The native renderer treats PDF/A and archival markers as document metadata and
thumbnail-rendering context. It does not validate PDF/A conformance.

## Supported Signals

- Catalog `/Metadata` presence is surfaced through document structure
  metadata.
- Bounded XMP inspection extracts common `pdfaid:part` and
  `pdfaid:conformance` markers.
- Catalog `/OutputIntents` presence is surfaced as archival metadata context.
- Embedded-file and associated-file style archive packets remain inert; file
  presence is exposed through metadata, not opened or previewed.

## Rendering Policy

Archival profile markers do not change rasterization by themselves. A PDF/A or
PDF/A-like document renders natively when its page content uses supported
graphics, text, image, font, and page-geometry features.

OutputIntents remain metadata/context for thumbnails and do not imply
color-managed proofing. Embedded files remain inert and do not affect page
painting.

## Explicit Non-Goals

- No PDF/A validator.
- No legal, records-management, or archival compliance certification.
- No XMP schema validation beyond bounded marker extraction.
- No attachment extraction, execution, or associated-file preview.
- No color-managed archival proofing or ICC conformance audit.

## Caller Guidance

Use `DocumentMetadata.archival` to identify PDF/A-style records and to display
their declared profile markers. Treat
`conformance_validation_performed = false` as a stable contract: native
metadata can classify markers, but applications that need compliance decisions
must run a dedicated validator outside the thumbnail renderer.
