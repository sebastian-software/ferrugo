# 0074: Annotation And Form Interaction Coverage

Status: done
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

Completed with the `test: add radio form appearance coverage` implementation
and the `docs: complete annotation form coverage` report update.

- Added selected and Off-state AcroForm radio widget fixtures.
- Added native-backend tests for `/AP /N` state-dictionary selection through
  `/AS` and visible On/Off rendering.
- Confirmed existing annotation, highlight, link, widget, checkbox, text-field,
  and signature-placeholder coverage remains green.
- Recorded form corpus comparison and remaining interactive/clip-scope limits in
  `docs/reports/annotation-form-coverage-2026-06-24.md`.
