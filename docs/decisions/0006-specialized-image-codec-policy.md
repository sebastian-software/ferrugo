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

## Deployment Matrix

| PDF image path | Desktop/server profile | WASM/low-memory profile | Policy |
| --- | --- | --- | --- |
| Raw Image XObject and inline images | Built in | Built in | Supported with existing raster and page pixel budgets. |
| `FlateDecode` image streams and PNG predictors | Built in | Built in | Supported through safe Rust stream decoding and predictor handling. |
| `DCTDecode`/`DCT` JPEG | Built in | Built in when the package includes the safe Rust JPEG decoder | Supported as the only specialized codec on the default native path. |
| Image masks and soft masks | Built in | Built in | Supported with image-byte and soft-mask depth budgets. |
| `CCITTFaxDecode`/`CCF` | Not bundled by default | Not bundled | Typed `image.filter` unsupported until a safe decoder slice is accepted. |
| `JPXDecode` | Not bundled by default | Not bundled | Typed `image.filter` unsupported until a budgeted pure-Rust or isolated decoder is selected. |
| `JBIG2Decode` | Not bundled by default | Not bundled | Typed `image.filter` unsupported; direct unsafe decoder bindings require separate security review. |

Server deployments should not reintroduce PDFium solely for specialized image
decoding. If a product needs CCITT, JPX, or JBIG2 before native support lands,
route that decision through an explicit out-of-process or sandboxed conversion
service with per-document policy, telemetry, and tenant-visible diagnostics.

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

Milestone 0209 promotes this into a deployment gate with
`fixtures/image-codec-deployment-manifest.tsv`: eight supported built-in image
paths must render natively, while CCITT, JBIG2, and JPX remain typed
`image.filter` boundaries.

## Follow-Up Criteria

A future codec implementation slice should proceed only when it has:

- corpus evidence that the codec materially improves common documents,
- a safe Rust decoder or a reviewed isolation boundary,
- explicit decoded-byte and output-pixel budgets,
- malformed-data tests,
- PDFium comparison fixtures for valid sample payloads,
- benchmark evidence for scan-heavy documents.
