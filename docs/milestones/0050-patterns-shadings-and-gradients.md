# 0050: Patterns Shadings And Gradients

Status: completed
Phase: 6
Size: medium
Depends on: 0049

## Goal

Render common pattern and shading fills that appear in reports, slides, and
browser-generated PDFs.

## Scope

- Support simple tiling patterns with bounded repeat counts.
- Support axial and radial shadings at thumbnail resolution.
- Cache sampled shading results where it reduces repeated work.
- Return typed unsupported errors for mesh shadings and complex patterns.

## Non-Goals

- Printer-grade gradient precision.
- Full mesh shading support.
- Infinite pattern recursion.

## Deliverables

- Pattern and shading render paths.
- Fixtures for tiling patterns and gradients.
- Sampling and recursion-limit tests.

## Acceptance Criteria

- Common gradient and simple pattern PDFs render recognizably.
- Pattern recursion and tile expansion are bounded.
- Unsupported shading types do not break the page render.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for shading fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed:

- First implementation slice adds direct `/Resources /Shading` parsing for
  `/ShadingType 2` axial shadings with `DeviceRGB` or `DeviceGray` Type 2
  sampled functions.
- Content-stream `sh` operators now produce shading display-list items when the
  named resource is supported, and unsupported shading types fail with typed
  errors.
- Axial shading rasterization projects device pixels onto the gradient axis and
  samples colors without per-pixel heap allocation.
- Native rendering now resolves page-level `/Shading` dictionaries for the path
  and form scan passes.
- Third implementation slice adds `/ShadingType 3` radial shading parsing and
  rasterization for the common concentric-circle case, sharing the same
  `DeviceRGB`/`DeviceGray` Type 2 function decoder and direct pixel sampling
  path.
- Second fixture slice adds generated `fixtures/generated/axial-gradient.pdf`
  through `scripts/generate_fixtures.py`, covering a full-page red-to-blue
  `DeviceRGB` axial shading.
- PDFium/native comparison for `axial-gradient.pdf` at `max-edge 120`:
  `120x120`, changed pixels `14400`, MAE `1.992`, p95 `3`, max channel delta
  `3`, native non-white pixels `14400`.
  Sample pixels: left PDFium `(224,0,31,255)` vs native `(222,0,33,255)`,
  center PDFium `(128,0,127,255)` vs native `(126,0,129,255)`, right PDFium
  `(33,0,222,255)` vs native `(31,0,224,255)`.
- Fourth fixture slice adds generated `fixtures/generated/radial-gradient.pdf`
  through `scripts/generate_fixtures.py`, covering a concentric white-to-blue
  `DeviceRGB` radial shading.
- PDFium/native comparison for `radial-gradient.pdf` at `max-edge 120`:
  `120x120`, changed pixels `12968`, MAE `1.854`, p95 `4`, max channel delta
  `5`, native non-white pixels `14400`.
  Sample pixels: center PDFium `(255,255,255,255)` vs native
  `(252,252,255,255)`, mid PDFium `(128,128,255,255)` vs native
  `(125,125,255,255)`, corner PDFium `(1,1,255,255)` vs native
  `(0,0,255,255)`.
- Fifth implementation slice adds colored tiling pattern support for
  `/PatternType 1`, `/PaintType 1` stream resources, fill color-space
  selection through `cs /Pattern`, pattern selection through `/Name scn`, and
  bounded repeated sampling in path fills.
- Sixth fixture slice adds generated `fixtures/generated/tiling-pattern.pdf`
  through `scripts/generate_fixtures.py`, covering repeated red/blue colored
  tiling-pattern fills.
- PDFium/native comparison for `tiling-pattern.pdf` at `max-edge 120`:
  `120x120`, exact pixel match with no changed pixels. Sample pixels at x
  `5/15/25/35/115`, y `60` match PDFium and native:
  `(255,0,0,255)`, `(0,0,255,255)`, `(255,0,0,255)`,
  `(0,0,255,255)`, `(0,0,255,255)`.
- Current validation:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check`
  - `cargo test --quiet`
  - `cargo test -p pdfrust-render pattern -- --nocapture`
  - `cargo test -p pdfrust-native tiling_pattern -- --nocapture`
  - `cargo test -p pdfrust-native axial_gradient -- --nocapture`
  - `cargo test -p pdfrust-native radial_gradient -- --nocapture`
  - `cargo test -p pdfrust-render shading -- --nocapture`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/axial-gradient.pdf --max-edge 120 --output target/pdfrust-thumbnails/axial-gradient-pdfium-0050.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/axial-gradient.pdf --max-edge 120 --output target/pdfrust-thumbnails/axial-gradient-native-0050.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/radial-gradient.pdf --max-edge 120 --output target/pdfrust-thumbnails/radial-gradient-pdfium-0050.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/radial-gradient.pdf --max-edge 120 --output target/pdfrust-thumbnails/radial-gradient-native-0050.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/tiling-pattern.pdf --max-edge 120 --output target/pdfrust-thumbnails/tiling-pattern-pdfium-0050.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/tiling-pattern.pdf --max-edge 120 --output target/pdfrust-thumbnails/tiling-pattern-native-0050.png`
  - `cargo clippy --all-targets --all-features -- -D warnings`
