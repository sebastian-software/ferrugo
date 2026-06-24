# 0053: AcroForm Appearance Rendering

Status: todo
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

Empty until done.
