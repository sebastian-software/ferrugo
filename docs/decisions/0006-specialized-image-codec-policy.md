# 0006: Specialized Image Codec Policy

Date: 2026-06-25.
Status: accepted.

## Context

Native rendering already supports unfiltered image samples, `FlateDecode` image
streams with PNG predictors, and a narrow safe `DCTDecode` JPEG path. Real-world
scanned PDFs also commonly use CCITT Fax, JPEG 2000, and JBIG2. These codecs
have different risk profiles from the current safe-Rust image path:

- CCITT Fax is common in monochrome scans and fax-style document archives.
- JPEG 2000 appears in scanned and office-exported PDFs but needs a decoder with
  a clear safety and budget story.
- JBIG2 is compact for bi-level scans and has a history of security-sensitive
  decoder bugs in native implementations.

## Decision

Keep the native renderer explicit and deterministic:

- `FlateDecode` and alias `/Fl` are supported through the existing stream
  decoder and predictor implementation.
- `DCTDecode` and alias `/DCT` are supported through the existing safe Rust JPEG
  decoder path.
- `CCITTFaxDecode` and alias `/CCF` remain unsupported in native rendering until
  a corpus-driven slice selects a safe decoder and validates row, K, EndOfLine,
  and byte-alignment behavior.
- `JPXDecode` remains unsupported until a pure-Rust or tightly isolated decoder
  is selected with memory and decompression budgets.
- `JBIG2Decode` remains unsupported until there is a sandboxed or otherwise
  strongly isolated decoder strategy. Do not add direct unsafe decoder bindings
  for JBIG2 without a separate safety review.

Unsupported specialized codecs must return `UnsupportedImageFilter` in the
render layer and map to the stable native feature bucket `image.filter`.

## Rationale

This keeps native-default behavior honest. Valid PDFs that need unsupported
codecs should not silently degrade to blank images or implicit PDFium fallback.
The caller can decide whether `image.filter` should route to PDFium.

The policy also keeps memory usage predictable. Codec adoption should come with
explicit decoded-size limits, decompression-ratio limits where applicable, and
fixture coverage before becoming a default native path.

## Current Corpus Evidence

Milestone 0089 adds deterministic generated fixtures for:

- `unsupported-ccitt-image.pdf`,
- `unsupported-jbig2-image.pdf`,
- `unsupported-jpx-image.pdf`.

The 0089 fallback summary reports 64 generated fixtures total, 59 native
renders, 4 fallbacks, and 1 encrypted error. Three fallbacks are the new
`image.filter` codec fixtures; the remaining fallback is the existing optional
content policy fixture.

## Follow-Up Criteria

A future codec implementation slice should proceed only when it has:

- corpus evidence that the codec materially improves common documents,
- a safe Rust decoder or a reviewed isolation boundary,
- explicit decoded-byte and output-pixel budgets,
- malformed-data tests,
- PDFium comparison fixtures for valid sample payloads,
- benchmark evidence for scan-heavy documents.
