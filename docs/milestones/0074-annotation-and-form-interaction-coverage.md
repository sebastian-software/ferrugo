# 0074: Annotation And Form Interaction Coverage

Status: todo
Phase: 12
Size: medium
Depends on: 0073

## Goal

Cover the annotation and form appearance patterns that remain common after basic
appearance rendering is available.

## Scope

- Test widget states, checkboxes, radio buttons, highlight annotations, and link
  borders in typical documents.
- Render static appearances according to documented policy.
- Preserve non-rendered interaction data for future layers without exposing a
  partial editing API.
- Add fallback reasons for unsupported dynamic appearances.

## Non-Goals

- Implement form filling or annotation editing.
- Execute JavaScript actions.
- Build an interactive PDF viewer.

## Deliverables

- Annotation and form state fixtures.
- Static appearance rendering improvements.
- Updated policy for unsupported interactive behavior.

## Acceptance Criteria

- Static form and annotation appearances render natively for common documents.
- Unsupported interactive behavior is not silently ignored when it affects
  visual output.
- Tests cover checked, unchecked, highlighted, and linked examples.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run form and annotation corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
