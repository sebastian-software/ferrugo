# 0051: Advanced Stroke And Clipping Fidelity

Status: in-progress
Phase: 6
Size: medium
Depends on: 0050

## Goal

Improve vector fidelity for real-world diagrams, charts, and thin-line office
content.

## Scope

- Implement line joins, caps, dash patterns, and miter limits consistently.
- Improve even-odd and nonzero clipping behavior.
- Handle hairlines and near-zero stroke widths with a documented thumbnail
  policy.
- Add reduced fixtures for charts, tables, and vector diagrams.

## Non-Goals

- Full CAD-grade vector precision.
- GPU acceleration.
- Arbitrary precision geometry.

## Deliverables

- Stroke expansion improvements.
- Clipping tests.
- Differential vector fixture comparisons.

## Acceptance Criteria

- Common chart and table lines remain visible and correctly clipped.
- Dash and join behavior is stable across thumbnail sizes.
- Geometry edge cases fail predictably instead of corrupting later drawing.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for vector fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First implementation slice adds bounded stroke dash-pattern state for the
  `d` operator. Dash arrays are stored in a fixed-size graphics-state
  representation, captured in path display-list items, and expanded into
  painted stroke segments once per rasterized path. Overlong dash arrays fail
  with a typed unsupported error instead of growing state allocations.
- Second fixture slice adds generated `fixtures/generated/dashed-stroke.pdf`
  through `scripts/generate_fixtures.py`, covering a horizontal `[10 10]`
  dashed vector stroke.
- PDFium/native comparison for `dashed-stroke.pdf` at `max-edge 120`:
  `120x120`, changed RGB pixels `80`, RGB MAE `1.2181`, p95 RGB delta `0`,
  max channel delta `255`, native non-white pixels `280`. Mid-dash and mid-gap
  samples at x `15/25/35/45/55/65`, y `60` match PDFium and native:
  black, white, black, white, black, white.
- Current validation:
  - `cargo test -p pdfrust-render dash -- --nocapture`
  - `cargo test -p pdfrust-native dashed_stroke -- --nocapture`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/dashed-stroke.pdf --max-edge 120 --output target/pdfrust-thumbnails/dashed-stroke-pdfium-0051.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/dashed-stroke.pdf --max-edge 120 --output target/pdfrust-thumbnails/dashed-stroke-native-0051.png`
