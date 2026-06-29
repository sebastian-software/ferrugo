# 0170: Raster Image Heavy Document Memory Gate

Status: done
Phase: 31
Size: medium
Depends on: 0169

## Goal

Harden native rendering for image-heavy PDFs such as scans, receipts, camera
captures, image-based reports, and mixed image/vector pages.

## Scope

- Add fixtures with many large images, repeated images, masks, rotations, and
  downsampled thumbnails.
- Audit decode cache, image reuse, color conversion, and raster allocation
  behavior.
- Add memory budget enforcement for image-heavy render paths.
- Improve diagnostics for images skipped or downsampled by policy.

## Non-Goals

- Add unbounded image cache growth for speed.
- Decode unsupported proprietary image formats without a policy decision.
- Sacrifice visual correctness for undocumented downsampling.

## Deliverables

- Image-heavy document fixture set.
- Memory profile report.
- Cache and downsampling policy updates.

## Acceptance Criteria

- Image-heavy documents stay within configured memory budgets.
- Repeated image resources are reused where safe.
- Unsupported image behavior is typed and documented.

## Validation

- Run native-only `cargo test`.
- Run image-heavy benchmark subset.
- Run memory profile for large image documents.
- Run visual comparison for accepted fixtures.

## Completion Notes

Completed on 2026-06-26.

Report:

- `docs/reports/raster-image-heavy-memory-gate-2026-06-26.md`

Implemented:

- Added `fixtures/image-heavy-memory-manifest.tsv` for repeated image XObject,
  rotated soft-mask, large scan, mixed compression, soft-mask, image-mask,
  predictor, and DCT/JPEG image-heavy coverage.
- Added `image-heavy-repeated-xobject-report.pdf` and
  `image-heavy-rotated-mask-sheet.pdf` generated fixtures.
- Added native default and low-memory tests for image-heavy render paths.

Validation:

- `cargo test -p ferrugo-native image_heavy -- --nocapture`
- Image-heavy supported gate: 8 total, 8 native rendered, 0 fallbacks, 0
  errors.
- Image-heavy benchmark: 8 total, 8 native rendered, 0 fallbacks, 0 errors, 0
  budget failures.
- Image-heavy low-memory benchmark: 8 total, 8 native rendered, 0 fallbacks, 0
  errors, 0 budget failures.
- Maintainer visual comparison: 8 total, 4 exact, 0 accepted drift, 4
  resampling fidelity blockers, 0 native errors, 0 PDFium errors.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
