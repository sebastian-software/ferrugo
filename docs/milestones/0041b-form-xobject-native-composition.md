# 0041b: Form XObject Native Composition

Status: done
Phase: 5
Size: small
Depends on: 0041a

## Goal

Wire existing Form XObject display-list support into the Rust-native thumbnail
renderer so simple form-backed pages render without requiring PDFium.

## Scope

- Resolve page-level Form XObject resources in `ferrugo-native`.
- Execute Form XObject path display lists before later image/text passes.
- Rasterize form paths into the existing page raster instead of allocating a
  separate output buffer.
- Keep Image XObject and Form XObject `Do` handling from failing each other.

## Non-Goals

- Render text or images inside Form XObjects.
- Preserve perfect mixed content order across page paths, forms, images, and
  text.
- Implement clipping enforcement for Form XObject bounding boxes.
- Replace the later 0059 facade parity gate.

## Deliverables

- Existing-device path rasterization helper.
- Native Form XObject resource resolution.
- Native backend coverage for `fixtures/generated/form-xobject.pdf`.
- Updated support matrix and architecture notes.

## Acceptance Criteria

- `form-xobject.pdf` renders non-white form content through `render-native`.
- Image XObject pages still render when the form pass sees `/Im* Do`.
- Missing image and missing form resources remain typed errors when the name is
  not known as another XObject subtype.
- The next font milestone depends on this pull-forward slice.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo run -p ferrugo-cli -- render-native fixtures/generated/form-xobject.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-form-native.png`.
- Confirm the native PNG contains non-white pixels.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `rasterize_paths_into` so form path display lists can paint into an
  existing page raster.
- Extended Image and Form XObject resource maps to remember known opposite
  subtype names, preventing `/Fm* Do` from failing the image pass and `/Im* Do`
  from failing the form pass.
- Wired page Form XObject resources into `ferrugo-native` and rasterized form
  path items before image and text passes.
- Added native backend tests for `form-xobject.pdf` and regression coverage for
  Image XObject rendering through the new form pass.
- Validation:
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test -p ferrugo-render -p ferrugo-native`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/form-xobject.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-form-native.png`
  - PNG probe: `dimensions=120x120 nonwhite=6400`,
    `bounds=20,8..99,87`,
    `sample_30_30=(51, 179, 77, 255)`,
    `sample_88_24=(51, 179, 77, 255)`.
