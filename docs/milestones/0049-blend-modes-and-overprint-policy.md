# 0049: Blend Modes And Overprint Policy

Status: done
Phase: 6
Size: medium
Depends on: 0048

## Goal

Implement the highest-value blend modes and define a practical overprint policy
for thumbnails.

## Scope

- Support normal, multiply, screen, and other corpus-driven blend modes.
- Keep blend operations branch-light and allocation-free per pixel.
- Define whether overprint is ignored, approximated, or unsupported.
- Add fixtures for blend-heavy browser and design-tool exports.

## Non-Goals

- Full print-production overprint fidelity.
- DeviceN spot color parity.
- Color-managed blending.

## Deliverables

- Blend-mode implementation for prioritized modes.
- Overprint policy documentation.
- Differential tests for supported blend modes.

## Acceptance Criteria

- Supported blend fixtures are visually close enough for thumbnails.
- Unsupported blend or overprint cases are explicit and stable.
- Blend operations do not allocate in the inner pixel loop.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for blend fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with:

- `9578902 feat: apply path blend modes`
- `c3a7140 feat: share blend states with form scans`

- First implementation slice adds direct `/ExtGState` resource parsing for
  `/BM` values `Normal`, `Compatible`, `Multiply`, and `Screen`.
- Path display-list execution applies `/gs` graphics-state resources and stores
  the active blend mode in each path item.
- Path rasterization applies `Normal`, `Multiply`, and `Screen` with
  allocation-free channel math inside the pixel loop.
- Enabled `/OP true` and `/op true` overprint policies fail with typed
  unsupported errors instead of silently rendering incorrect thumbnails.
- Native rendering now resolves page-level `/Resources /ExtGState` dictionaries
  for the path pass.
- Second fixture slice adds generated `fixtures/generated/blend-modes.pdf`
  through `scripts/generate_fixtures.py`, covering a gray backdrop with red
  `Multiply` and blue `Screen` path fills.
- The Form XObject scan path now accepts the same page-level `/ExtGState`
  resources as the primary path pass, so repeated content scans do not reject
  otherwise supported blend-mode content.
- PDFium/native comparison for `blend-modes.pdf` at `max-edge 120`:
  `120x120`, changed pixels `0`, MAE `0.000`, p95 `0`, max channel delta `0`,
  native non-white pixels `14400`.
  Sample pixels: background `(128,128,128,255)`, multiply `(128,0,0,255)`,
  screen `(128,128,255,255)`.
- Current validation:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check`
  - `cargo test --quiet`
  - `cargo test -p ferrugo-native blend_modes -- --nocapture`
  - `cargo test -p ferrugo-render ext_graphics_state -- --nocapture`
  - `cargo test -p ferrugo-render blend_source_with_backdrop -- --nocapture`
  - `cargo test -p ferrugo-render display_list_should_apply_external_graphics_state_blend_mode -- --nocapture`
  - `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render fixtures/generated/blend-modes.pdf --max-edge 120 --output target/ferrugo-thumbnails/blend-modes-pdfium-0049.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/blend-modes.pdf --max-edge 120 --output target/ferrugo-thumbnails/blend-modes-native-0049.png`
  - `cargo clippy --all-targets --all-features -- -D warnings`
