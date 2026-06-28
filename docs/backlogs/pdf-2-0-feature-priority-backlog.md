# PDF 2.0 Feature Priority Backlog

Status: accepted for 0181.
Date: 2026-06-28.

This backlog converts the PDF 2.0 usage classifier into 1.2 roadmap priorities.
Counts come from `target/pdf20-0181-usage.json`.

## Priority Slices

| Rank | Slice | Evidence | Recommendation | Validation gate |
| ---: | --- | --- | --- | --- |
| 1 | Black point compensation | 1 PDF 2.0 document, `graphics.color-management` typed unsupported. | Keep unsupported unless real-corpus frequency or customer samples justify native color-management work. | Focused fixture plus visual threshold policy before changing behavior. |
| 2 | Associated files | 1 PDF 2.0 document, native rendered, metadata-only for thumbnails. | Keep accepted and ensure attachments do not affect pixels or leak private payload bytes into diagnostics. | Native render gate plus metadata/privacy report check. |
| 3 | Catalog `/Version /2.0` | 3 PDF 2.0 documents, all detected by header and catalog version. | Continue accepting version markers when page operators use supported paths. | PDF 2.0 supported-family fallback gate. |

## Deferred Features

The corpus does not yet contain representative samples for PDF 2.0 features
that may affect layers, transparency, annotations, security, or color beyond
`/UseBlackPtComp`. Add reduced fixtures before implementing or approximating
those semantics.

## Release Rule

PDF 2.0 feature work for 1.2 must be ranked by observed corpus frequency and
visual impact. Unknown or low-frequency visual semantics should return stable
typed unsupported buckets instead of reintroducing PDFium runtime fallback.
