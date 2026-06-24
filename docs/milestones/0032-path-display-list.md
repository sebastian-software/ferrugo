# 0032: Path Display List

Status: done
Phase: 2
Size: medium
Depends on: 0031

## Goal

Convert basic path construction and painting operators into a display list.

## Scope

- Interpret `m`, `l`, `c`, `h`, `re`, `S`, `s`, `f`, `F`, `f*`, `B`, and
  `B*` where needed by fixtures.
- Store path segments in a Rust-native display list.
- Capture fill and stroke state at paint time.
- Add path complexity limits.

## Non-Goals

- Rasterize paths.
- Implement gradients or patterns.
- Implement clipping beyond storing a placeholder command.

## Deliverables

- Path command representation.
- Display-list path items.
- Tests for simple vector fixtures.

## Acceptance Criteria

- Generated vector PDFs produce inspectable display-list path items.
- Unsupported path operators fail predictably.
- Path memory use is bounded by explicit limits.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare display-list dimensions against PDFium-rendered page metadata.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `feat: build path display lists` change.

- Added bounded display-list types in `pdfrust-render`: `DisplayList`,
  `DisplayItem`, `PathDisplayItem`, `PathSegment`, `PaintMode`, `FillRule`,
  `PathBounds`, and `DisplayListOptions`.
- Added `build_path_display_list` for supported path construction and painting
  operators: `m`, `l`, `c`, `h`, `re`, `S`, `s`, `f`, `F`, `f*`, `B`, `B*`,
  `W`, `W*`, and `n`.
- Extended graphics-state color capture to `DeviceColor::Rgb` for generated
  vector fixtures using `RG` and `rg`.
- Added explicit path-segment and display-item limits with typed overflow
  errors.
- Added tests for stroked paths, rectangle fills, clipping placeholders, the
  generated `vector-paths.pdf` fixture, unsupported path operators, missing
  current point errors, and both complexity limits.
- Validation:
  - `cargo fmt --check`
  - `cargo test -p pdfrust-render`
