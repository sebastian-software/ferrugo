# 0137: Image Downsampling And Color Conversion Optimization

Status: done
Phase: 25
Size: medium
Depends on: 0136

## Goal

Optimize image-heavy native rendering by reducing unnecessary decoded memory,
copying, and color conversion work for thumbnail-sized outputs.

## Scope

- Profile common scan, photo, and office-export image paths.
- Add downsampling or decode-window decisions where supported safely.
- Reduce avoidable intermediate allocations during color conversion.
- Keep output deterministic and visually compared against existing thresholds.

## Non-Goals

- Rewrite every codec backend.
- Add lossy behavior that changes full-resolution semantics silently.
- Support unsupported specialized codecs in this optimization slice.

## Deliverables

- Image optimization report.
- Benchmarks for scan, photo, and mixed image documents.
- Regression tests for color conversion and alpha behavior.

## Acceptance Criteria

- Thumbnail rendering avoids decoding more image data than necessary where
  practical.
- Memory and runtime improve for image-heavy fixtures.
- Visual output remains within documented drift thresholds.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run image-heavy visual comparisons.
- Run memory and runtime benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

- Added in-place PNG predictor reversal for decoded Flate image samples,
  avoiding a second full decoded-sample allocation for predictor images.
- Added a per-draw single-entry image sample cache so repeated thumbnail target
  pixels that map to the same source pixel reuse converted RGBA samples.
- Added a focused CMYK plus soft-mask cache regression test and kept the
  existing color conversion, alpha, image mask, Indexed, DCT, and predictor
  image tests green.
- Wrote
  `docs/reports/image-downsampling-color-optimization-2026-06-26.md` with
  benchmark and visual comparison evidence.
- Validation:
  `cargo fmt --check`;
  `cargo test -p pdfrust-render image_ -- --nocapture`;
  `cargo check --workspace`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo test --workspace`;
  `cargo test --workspace --no-default-features`;
  image-heavy native benchmark at `target/image-0137-final-benchmark.json`;
  PDFium visual comparison at `target/image-0137-visual-diff.json`.

Visual comparison completed without render errors, but the known PDFium
resampling parity blockers for mobile scan rotation/crop/mixed compression
remain out of scope for this optimization slice.
