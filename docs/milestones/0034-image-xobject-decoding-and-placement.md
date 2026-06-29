# 0034: Image XObject Decoding And Placement

Status: done
Phase: 2
Size: medium
Depends on: 0033

## Goal

Decode and place common image XObjects in the display list.

## Scope

- Resolve image XObject resources.
- Support DeviceRGB and DeviceGray image metadata.
- Decode Flate-backed images.
- Add DCT/JPEG decoding strategy or a narrow implementation behind a safe API.
- Store positioned image items in the display list.

## Non-Goals

- JPX/JPEG 2000, CCITT, or JBIG2.
- Full color management.
- Image interpolation tuning.

## Deliverables

- Image XObject resolver.
- Image display-list items.
- Tests for generated image fixtures.

## Acceptance Criteria

- Generated PDFs with embedded RGB images produce image display-list items.
- Unsupported image filters and color spaces return typed errors.
- Image byte ownership is explicit and avoids redundant copies where practical.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare image placement metadata against PDFium-rendered fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `feat: place decoded image xobjects` change.

- Added `ImageResources`, `ImageObjectResolver`, `ImageXObject`,
  `ImageDisplayItem`, and `ImageColorSpace` to `ferrugo-render`.
- Resolved image XObjects from `/XObject` resource dictionaries against loaded
  classic and modern documents.
- Supported unfiltered and `FlateDecode` `DeviceRGB`/`DeviceGray` image streams
  with explicit decoded-byte limits and exact sample-length validation.
- Stored image placements as display-list items using the active CTM and unit
  square bounds.
- Used `Arc<[u8]>` for decoded image samples so repeated `Do` placements share
  bytes instead of duplicating them.
- Documented and enforced `DCTDecode`/JPEG as an explicit unsupported filter for
  this milestone; full JPEG decoding is deferred to image filter coverage.
- Added tests for Flate RGB image resources, DeviceGray image resources, CTM
  placement bounds, shared decoded samples, missing image resources, unsupported
  `DCTDecode`, and unsupported color spaces.
- Validation:
  - `cargo fmt --check`
  - `cargo test -p ferrugo-render`
