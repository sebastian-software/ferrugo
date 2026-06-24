# 0054: Optional Content And Layer Policy

Status: todo
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

Empty until done.
