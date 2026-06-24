# Rust Native Crate Architecture

Status: accepted initial layout.
Date: 2026-06-24.

The Rust-native renderer grows behind the existing backend-neutral
`pdfrust-thumbnail` facade. PDFium remains the behavior oracle and differential
baseline, but the Rust-native crates must not expose PDFium handles, symbols, or
naming as public API.

## Crates

| Crate | Role | Depends On |
| --- | --- | --- |
| `pdfrust-syntax` | Byte-level PDF syntax, tokens, and offset-aware parser errors. | none |
| `pdfrust-object` | Indirect objects, references, xref data, trailers, catalog, and page tree. | `pdfrust-syntax` |
| `pdfrust-content` | Page content stream tokenization and display-list interpretation. | `pdfrust-object` |
| `pdfrust-render` | Raster buffers, page transforms, drawing, and pixel output helpers. | `pdfrust-content`, `pdfrust-thumbnail` |
| `pdfrust-native` | Rust-native backend adapter for the thumbnail facade. | `pdfrust-object`, `pdfrust-render`, `pdfrust-thumbnail` |

## Dependency Direction

```text
pdfrust-thumbnail
        ^
        |
pdfrust-native -----> pdfrust-render -----> pdfrust-content
        |                                      |
        +---------------> pdfrust-object <-----+
                             |
                             v
                       pdfrust-syntax
```

The direction is intentionally one-way from high-level backend code down to
syntax and object loading. Lower layers do not know about the PDFium backend,
CLI process model, Node packaging, or product distribution.

## Safety Defaults

Rust-native implementation crates start with `#![forbid(unsafe_code)]`.
Performance work can add isolated unsafe modules only after correctness,
profiling, and review justify it. The default implementation style is borrowed
input, typed IDs, checked offsets, bounded decoding, and explicit error values.

## Milestone Boundary

This layout milestone does not parse or render PDFs. It creates stable crate
ownership boundaries so milestones 0022 and later can add behavior in small,
measurable slices against the PDFium baseline.

## Current Syntax Foundation

`pdfrust-syntax` owns borrowed PDF input and offset-aware syntax failures.
`PdfBytes<'a>` and `ByteCursor<'a>` keep scanning over borrowed bytes, while
`ByteOffset`, `SyntaxErrorKind`, and `SyntaxError` provide diagnostics that
later parser layers can preserve without inventing new error plumbing.
