# 0054: Optional Content And Layer Policy

Status: in-progress
Phase: 7
Size: medium
Depends on: 0053

## Goal

Handle optional content groups predictably so layered PDFs render the expected
default thumbnail.

## Scope

- Parse optional content properties from the document catalog.
- Apply the default layer visibility state during content interpretation.
- Ignore or report unsupported usage applications consistently.
- Add fixtures for simple layer-on and layer-off PDFs.

## Non-Goals

- User-selectable layer toggles.
- Full optional content intent handling.
- Interactive viewer preferences.

## Deliverables

- Optional content visibility resolver.
- Layered PDF fixtures.
- Documentation for unsupported optional content behavior.

## Acceptance Criteria

- Default-visible layers render and default-hidden layers stay hidden.
- Unknown optional content policies do not silently render misleading output.
- Layer decisions are observable in diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential comparisons for layer fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First implementation slice resolves simple catalog `/OCProperties /D`
  defaults, page `/Resources /Properties` OCG references, and `/OC ... BDC`
  marked-content sections before the existing native display-list passes run.
- Generated `fixtures/generated/optional-content-layer-on.pdf` and
  `fixtures/generated/optional-content-layer-off.pdf` cover the same marked
  rectangle with default-visible and default-hidden layer configuration.
- PDFium/native comparison at `max-edge 120`: `optional-content-layer-on.pdf`
  renders `3200` non-white pixels in both backends, with the optional red
  rectangle visible; `optional-content-layer-off.pdf` renders `1600` non-white
  pixels in both backends, with the optional rectangle hidden.
- Current validation:
  - `cargo test -p pdfrust-native optional_content_layer -- --nocapture`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/optional-content-layer-on.pdf --max-edge 120 --output target/pdfrust-thumbnails/optional-content-layer-on-pdfium-0054.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/optional-content-layer-on.pdf --max-edge 120 --output target/pdfrust-thumbnails/optional-content-layer-on-native-0054.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/optional-content-layer-off.pdf --max-edge 120 --output target/pdfrust-thumbnails/optional-content-layer-off-pdfium-0054.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/optional-content-layer-off.pdf --max-edge 120 --output target/pdfrust-thumbnails/optional-content-layer-off-native-0054.png`
