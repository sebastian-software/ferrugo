# Codec And Transparency Boundary Gate

Date: 2026-07-04
Issues: #67, #65

## Summary

This report closes the immediate post-0.3.0 release-boundary work for scan
codecs and advanced transparency. It does not add a new decoder or a broader
blend implementation. Instead it makes the scoped release behavior executable:
supported image/transparency families must still render natively, and deferred
specialized codecs plus advanced transparency boundaries must still produce
typed, stable fallback buckets.

## Codec Decision

`CCITTFaxDecode`/`CCF` is the first future codec implementation candidate when
the project accepts a decoder slice, because it maps to common monochrome
fax/archive scans and has a smaller format surface than JPX or JBIG2.

The current native runtime keeps these codecs deferred:

| Codec | Current behavior | Future requirement |
| --- | --- | --- |
| `CCITTFaxDecode`/`CCF` | Typed `image.filter` fallback. | Safe decoder slice with row/K/EOL/byte-alignment fixtures, malformed-data tests, decoded-byte budgets, benchmarks, and fuzz smoke. |
| `JPXDecode` | Typed `image.filter` fallback. | Pure-Rust or tightly isolated decoder with memory/decompression budgets. |
| `JBIG2Decode` | Typed `image.filter` fallback. | Sandboxed or strongly isolated decoder strategy plus separate safety review. |

No direct unsafe decoder binding is accepted for the default native runtime by
this decision.

## Transparency Decision

The current native runtime keeps the already-supported transparency surface:
ExtGState alpha, isolated groups, knockout metadata, Multiply/Screen blend modes,
blend-mode array fallback to a supported entry, and image soft masks.

These boundaries remain explicitly deferred:

| Boundary | Current behavior | Future requirement |
| --- | --- | --- |
| ExtGState luminosity soft mask | Typed `graphics.transparency` fallback. | One-semantic fixture slice with visual oracle evidence and bounded intermediate surfaces. |
| Overlay and advanced blend modes | Typed `graphics.transparency` fallback. | One-mode-at-a-time implementation with focused visual comparison and memory/profile gates. |

The decision avoids a new fallback bucket, avoids generic errors, and keeps
intermediate transparency surfaces under the existing renderer budgets.

## Gate

The durable local gate is:

```sh
bash scripts/check_codec_transparency_boundaries.sh
```

It writes:

- `target/codec-transparency-boundaries/codec-transparency-boundaries-summary.txt`
- `target/codec-transparency-boundaries/image-codec-supported.json`
- `target/codec-transparency-boundaries/image-codec-unsupported.json`
- `target/codec-transparency-boundaries/transparency-supported.json`
- `target/codec-transparency-boundaries/transparency-unsupported.json`
- `target/codec-transparency-boundaries/image-codec-benchmark.json`
- `target/codec-transparency-boundaries/transparency-low-memory-benchmark.json`
- `target/codec-transparency-boundaries/transparency-poppler-visual-diff.json`

The script asserts that:

- supported image-codec deployment families render with zero fallbacks;
- CCITT/JBIG2/JPX produce exactly three typed `image.filter` fallbacks;
- supported transparency conformance families render with zero fallbacks;
- luminosity soft-mask and Overlay fixtures produce exactly two typed
  `graphics.transparency` fallbacks;
- image-heavy and transparency-stack benchmark/profile gates remain bounded;
- focused transparency visual comparison over group/blend/image-soft-mask rows
  stays within the documented review threshold;
- the fuzz smoke gate still passes.

## Validation

Commands run for this PR:

```sh
bash scripts/check_codec_transparency_boundaries.sh
cargo fmt --check
git diff --check
```
