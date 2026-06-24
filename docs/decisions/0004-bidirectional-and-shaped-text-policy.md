# 0004: Bidirectional And Shaped Text Policy

Status: accepted
Date: 2026-06-24

## Context

PDF text streams normally contain glyph character codes plus explicit text
matrices, advances, and positioning operators. For thumbnails, the renderer
should follow those PDF rendering instructions rather than reshaping Unicode
source text.

## Decision

The native renderer treats bidirectional and shaped text as pre-shaped PDF glyph
data when the document provides positioned glyph codes and a supported mapping
layer such as ToUnicode. It will not add a Unicode shaping dependency for source
text in this phase.

Unsupported cases should stay explicit and typed at the PDF feature boundary:
unsupported encodings, unsupported CMap constructs, missing glyph outlines, or
font-program gaps.

## Consequences

- Pre-positioned Arabic, Hebrew, Indic, and shaped Latin fixture streams can
  render natively through the existing text display-list path.
- Visual quality remains limited by the fallback glyph rasterizer until real
  glyph outlines and font-backed painting are used.
- Heavy shaping libraries require separate benchmark and corpus evidence before
  being added.
