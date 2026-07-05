# Codec And Transparency Boundary Gate

Date: 2026-07-04
Issues: #67, #65

## Summary

This report closes the immediate post-0.3.0 release-boundary work for scan
codecs and advanced transparency. Issue 103 later adds the first native scan
decoder slice for CCITT Fax. The executable behavior is now: supported
image/transparency families, including CCITT Group 3/4 fixtures, must render
natively, while deferred JPX/JBIG2 codecs plus advanced transparency boundaries
must still produce typed, stable fallback buckets.

## Codec Decision

`CCITTFaxDecode`/`CCF` was the first codec implementation candidate because it
maps to common monochrome fax/archive scans and has a smaller format surface
than JPX or JBIG2. The native runtime now decodes Group 3 1D, mixed Group 3
1D/2D, and Group 4 CCITT images with decoded-byte budgets.

The current native runtime keeps these codecs deferred:

| Codec | Current behavior | Future requirement |
| --- | --- | --- |
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

- supported image-codec deployment families, including CCITT, render with zero
  fallbacks;
- JPX/JBIG2 produce exactly two typed `image.filter` fallbacks;
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
