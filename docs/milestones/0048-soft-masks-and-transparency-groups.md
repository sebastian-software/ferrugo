# 0048: Soft Masks And Transparency Groups

Status: in-progress
Phase: 6
Size: medium
Depends on: 0047

## Goal

Support common alpha and transparency constructs that affect modern generated
PDF thumbnails.

## Scope

- Parse and apply soft masks for images where the mask format is supported.
- Render isolated transparency groups into bounded intermediate buffers.
- Composite group results back into the page raster buffer.
- Add memory budgets for nested transparency rendering.

## Non-Goals

- Full blend-mode parity.
- Full PDF transparency model coverage.
- Unbounded nested group rendering.

## Deliverables

- Soft-mask application path.
- Transparency group render path with budgets.
- Alpha fixture pixel comparisons.

## Acceptance Criteria

- Common transparent image and group fixtures render recognizably.
- Nested or oversized groups fail with typed budget errors.
- Intermediate buffers are reused or bounded where practical.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for transparency fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First implementation slice adds `/SMask` image support for referenced 8-bit
  DeviceGray Image XObjects with dimensions matching the parent image.
- Soft-mask sample data is stored beside the decoded image samples as
  `Arc<[u8]>`, so repeated image placements share both color and alpha buffers.
- Added a dedicated soft-mask depth budget. Nested soft masks now fail with
  `SoftMaskDepthOverflow`, while unsupported mask forms fail with
  `UnsupportedSoftMask`.
- The image raster path keeps the direct overwrite fast path for opaque pixels
  and uses source-over compositing only for pixels whose soft-mask alpha is
  below `255`.
- Validation so far: `cargo fmt --check`, `git diff --check`,
  `cargo test -p pdfrust-render soft_mask -- --nocapture`, `cargo check`,
  `cargo test --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`.
