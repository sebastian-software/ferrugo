# 0035: Form XObject Recursion And Budgets

Status: done
Phase: 2
Size: medium
Depends on: 0034

## Goal

Interpret Form XObjects with explicit recursion and resource budgets.

## Scope

- Resolve Form XObject resources.
- Apply form matrices and bounding boxes.
- Reuse the content interpreter for nested form content.
- Enforce recursion-depth and display-list-size limits.

## Non-Goals

- Transparency groups.
- Soft masks.
- Pattern rendering.

## Deliverables

- Form XObject interpreter path.
- Budget configuration.
- Tests for nested forms and recursion failures.

## Acceptance Criteria

- Generated Form XObject fixtures produce nested display-list output.
- Recursive or oversized forms fail safely.
- Resource inheritance behavior is documented and tested.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare simple form fixture output against PDFium dimensions or pixels where
  available.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `FormResources`, `FormXObject`, and `build_form_display_list` in
  `ferrugo-render`.
- Form resolution now walks page-level `/XObject` dictionaries and nested local
  form `/Resources /XObject` dictionaries by indirect reference, so local names
  can resolve nested forms that are not exposed on the page.
- Form execution applies the caller CTM plus the form `/Matrix`, emits a
  bounding-box clip placeholder from `/BBox`, and recursively reuses the path
  display-list interpreter.
- Resource inheritance policy: forms without `/Resources` inherit the caller
  XObject scope; forms with local `/Resources /XObject` use local form
  references for nested `Do` invocations.
- Added `max_form_recursion_depth` to `DisplayListOptions` with a default depth
  limit of 16 and typed `FormRecursionOverflow`, `MissingForm`,
  `MissingFormObject`, and `InvalidFormResource` failures.
- Added generated `fixtures/generated/form-xobject.pdf` through
  `scripts/generate_fixtures.py`.
- Added tests for generated form fixtures, form matrix and BBox handling, local
  nested form resources, missing form resources, and recursion-limit failures.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- compare-metadata fixtures/generated/form-xobject.pdf --output target/ferrugo-thumbnails/form-xobject-metadata-comparison.json`
    produced `status: match` with one 120x120 page for both PDFium and
    Rust-native.
  - `cargo clippy --all-targets --all-features -- -D warnings`
