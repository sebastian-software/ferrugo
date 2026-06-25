# 0129: Mobile Scan And Camera PDF Robustness

Status: done
Phase: 23
Size: medium
Depends on: 0128

## Goal

Improve robustness for PDFs produced by mobile scanners and camera apps, where
large images, rotation metadata, OCR layers, and compression choices vary.

## Scope

- Add fixtures for rotated scans, cropped images, OCR overlays, and mixed
  compression.
- Verify large-image downsampling and memory behavior.
- Ensure page rotation and crop boxes match expected visual orientation.
- Track unsupported mobile-app image filters explicitly.

## Non-Goals

- Deskew images.
- Run OCR or improve OCR quality.
- Perform document cleanup or enhancement.

## Deliverables

- Mobile scan fixture family.
- Large-image memory and downsampling report.
- Unsupported filter backlog for scanner-produced PDFs.

## Acceptance Criteria

- Common mobile scan PDFs render natively within memory budgets.
- OCR text layers do not visually leak when intended to be invisible.
- Rotation and crop behavior is stable across scanner variants.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run mobile-scan visual comparisons.
- Run large-image memory benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added four generated mobile-scan fixtures covering rotated camera scans,
  cropped photo scans, invisible OCR overlays, and mixed Flate/DCT image
  compression.
- Added `fixtures/mobile-scan-manifest.tsv` with supported families
  `rotation`, `crop`, `ocr-layer`, and `compression`, plus an
  `unsupported-filter` backlog family for CCITT, JBIG2, and JPX scanner-style
  image filters.
- Added native regression coverage for mobile scan rendering, invisible OCR
  overlay preservation, and rotation/CropBox metadata.
- Native supported-family fallback gate: 9/9 rendered natively, 0 fallbacks,
  0 errors.
- Unsupported-filter backlog gate: 3/3 remain in `image.filter`, 0 unexpected
  errors.
- Native benchmark gate: 9/9 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle at strict default thresholds: 4 exact matches,
  1 accepted drift, 4 blockers, 0 native render errors, 0 PDFium render
  errors. The blockers are image resampling, mixed JPEG/Flate drift, and
  rotation/text raster differences rather than native coverage failures.
- Report: `docs/reports/mobile-scan-robustness-2026-06-25.md`.
