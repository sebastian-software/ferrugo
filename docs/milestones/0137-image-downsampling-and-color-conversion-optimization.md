# 0137: Image Downsampling And Color Conversion Optimization

Status: todo
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

Empty until done.
