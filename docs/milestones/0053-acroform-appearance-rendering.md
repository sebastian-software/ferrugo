# 0053: AcroForm Appearance Rendering

Status: in-progress
Phase: 7
Size: medium
Depends on: 0052

## Goal

Render common AcroForm field appearances without implementing an interactive
form engine.

## Scope

- Resolve AcroForm resources and widget annotations.
- Render existing field appearance streams.
- Generate simple fallback appearances only if corpus data shows high value.
- Keep form field values and scripts non-executable.

## Non-Goals

- Editing forms.
- Calculating form JavaScript.
- XFA support.

## Deliverables

- AcroForm appearance render path.
- Fixtures for text fields, checkboxes, and signature placeholders.
- Documentation for unsupported interactive form behavior.

## Acceptance Criteria

- Common filled form PDFs show visible field contents when appearances exist.
- Missing appearance generation policy is documented.
- Form scripts are never executed.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for form fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First fixture slice adds generated `fixtures/generated/acroform-text-field.pdf`
  through `scripts/generate_fixtures.py`, covering a catalog `/AcroForm` with a
  `/Fields` text-field widget that has an existing normal appearance stream.
  Native rendering uses the static widget appearance path from 0052; no form
  values, default appearances, JavaScript, or field calculations are executed.
- PDFium bridge probe for `acroform-text-field.pdf` at `max-edge 140` stays
  blank through the plain `FPDF_RenderPageBitmap` path, while native renders
  `1200` non-white pixels. Keep this as native regression coverage until a
  form-fill-aware PDFium oracle path exists.
- Checkbox fixture slice adds generated `fixtures/generated/acroform-checkbox.pdf`
  with a `/Btn` widget field, `/V /Yes`, `/AS /Yes`, and a normal appearance
  state dictionary containing `/Yes` and `/Off` form XObjects. Native rendering
  verifies that the checked-state appearance is selected without evaluating
  form values or scripts.
- PDFium bridge probe for `acroform-checkbox.pdf` at `max-edge 80` also stays
  blank through the plain `FPDF_RenderPageBitmap` path, while native renders
  `140` non-white pixels.
- Signature placeholder fixture slice adds generated
  `fixtures/generated/acroform-signature-placeholder.pdf` with a `/Sig` widget
  field and static normal appearance stream. Native rendering treats it as a
  non-interactive appearance only; no signature validation or field calculation
  runs.
- PDFium bridge probe for `acroform-signature-placeholder.pdf` at `max-edge 160`
  stays blank through the plain `FPDF_RenderPageBitmap` path, while native
  renders `3000` non-white pixels.
- Current validation:
  - `cargo test -p pdfrust-native acroform_text_field -- --nocapture`
  - `cargo test -p pdfrust-native acroform_checkbox -- --nocapture`
  - `cargo test -p pdfrust-native acroform_signature -- --nocapture`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/acroform-text-field.pdf --max-edge 140 --output target/pdfrust-thumbnails/acroform-text-field-pdfium-0053.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/acroform-text-field.pdf --max-edge 140 --output target/pdfrust-thumbnails/acroform-text-field-native-0053.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/acroform-checkbox.pdf --max-edge 80 --output target/pdfrust-thumbnails/acroform-checkbox-pdfium-0053.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/acroform-checkbox.pdf --max-edge 80 --output target/pdfrust-thumbnails/acroform-checkbox-native-0053.png`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/acroform-signature-placeholder.pdf --max-edge 160 --output target/pdfrust-thumbnails/acroform-signature-placeholder-pdfium-0053.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/acroform-signature-placeholder.pdf --max-edge 160 --output target/pdfrust-thumbnails/acroform-signature-placeholder-native-0053.png`
