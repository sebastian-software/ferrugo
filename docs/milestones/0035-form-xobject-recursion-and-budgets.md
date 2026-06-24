# 0035: Form XObject Recursion And Budgets

Status: todo
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

Empty until done.
