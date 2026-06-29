# 0041a: Inline Image Stream Execution

Status: done
Phase: 5
Size: small
Depends on: 0041

## Goal

Render simple inline image streams in the Rust-native backend so generated
browser/report-style thumbnails do not silently drop `BI`/`ID`/`EI` image data.

## Scope

- Tokenize `BI`/`ID`/`EI` inline image objects without treating raw image bytes
  as normal content operators.
- Decode bounded 8-bit unfiltered `DeviceRGB`/`DeviceGray` inline image data.
- Reuse the existing image display-list and rasterization path.
- Keep unsupported inline filters explicit.

## Non-Goals

- Decode JPEG, CCITT, JPX, JBIG2, or predictor-filtered inline images.
- Implement decode arrays or broader color management.
- Handle every ambiguous `EI` byte sequence inside compressed image data.
- Compose Form XObjects in the combined native render path.

## Deliverables

- Inline image content token.
- Inline image display-list execution.
- Native backend coverage for `fixtures/generated/inline-image.pdf`.
- Updated renderer support documentation.

## Acceptance Criteria

- `inline-image.pdf` renders non-white colored pixels through `render-native`.
- Inline image sample data is bounded by the existing image byte budget.
- Unsupported inline image filters fail with typed unsupported diagnostics.
- Path and text interpreters ignore inline image tokens without corrupting
  operand state.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo run -p ferrugo-cli -- render-native fixtures/generated/inline-image.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-inline-image-native.png`.
- Confirm the native PNG contains non-white pixels.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `ContentToken::InlineImage` so `ferrugo-content` parses inline image
  dictionaries and raw image bytes as one token.
- Added unfiltered 8-bit `DeviceRGB`/`DeviceGray` inline image execution in
  `ferrugo-render`, reusing `DisplayItem::Image` and `rasterize_images`.
- Added native backend test coverage proving `inline-image.pdf` now renders
  the expected RGB quadrants.
- Left filtered inline images as explicit `UnsupportedImageFilter` cases for
  0047.
- Validation:
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test -p ferrugo-content -p ferrugo-render -p ferrugo-native`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/inline-image.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-inline-image-native.png`
  - PNG probe: `dimensions=120x120 nonwhite=4096`,
    `sample_44_44=(255, 0, 0, 255)`,
    `sample_76_44=(0, 255, 0, 255)`,
    `sample_44_76=(0, 0, 255, 255)`.
