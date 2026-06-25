# Document Metadata Policy

Status: accepted for 0094.
Date: 2026-06-25.

Native metadata inspection is a non-rendering API surface. It should let
callers avoid PDFium for common document inspection without treating metadata
success as a rendering prerequisite.

## Supported

- Page count and per-page visible size.
- Classic trailer `/Info` references for common document information fields:
  title, author, subject, keywords, creator, producer, creation date, and
  modification date.
- Catalog XMP presence through `/Metadata`.
- Tagged-PDF presence through `/MarkInfo` and `/StructTreeRoot`.
- Named-destination presence through catalog `/Dests` or `/Names /Dests`.
- Outline presence and bounded outline item counting.
- Direct page-label number trees with decimal, roman, alphabetic, prefix, and
  start-number support.

## Unsupported

- Accessibility tree extraction or role-map interpretation.
- Full XMP packet parsing.
- Viewer UI behavior for outlines or named destinations.
- Name-tree traversal beyond the direct common `/Nums` page-label form.
- PDFium parity for extended metadata fields; PDFium remains a page
  count/size oracle in `compare-metadata`.

## Bounds

Outline traversal stops after `256` reached items. Page label expansion stops
after `4096` pages and reports truncation. Cycles in outline references are
ignored after the first visit to keep inspection bounded.

## Error Behavior

Malformed required metadata structures fail native inspection with the stable
`malformed` error class. Missing optional metadata produces default empty
fields instead of an error.
