# 0207: Annotation Popup Stamp And FreeText Fidelity

Status: todo
Phase: 39
Size: medium
Depends on: 0206

## Goal

Improve Rust-native fidelity for common review, markup, stamp, popup, and
FreeText annotations found in legal, government, and business workflows.

## Scope

- Add fixtures for stamps, FreeText boxes, popups, highlights, comments,
  callouts, and print-visible annotation states.
- Validate annotation appearance streams, default appearances, opacity,
  rotation, page boxes, and print-preview behavior.
- Track unsupported annotation behavior through typed diagnostics.
- Keep interactive state handling separate from static render fidelity.

## Non-Goals

- Build a complete annotation editor.
- Synchronize collaborative review comments.
- Render JavaScript-driven annotation behavior.

## Deliverables

- Annotation fidelity corpus.
- Print-preview and screen-rendering comparison report.
- Unsupported annotation taxonomy update.

## Acceptance Criteria

- Common markup annotations render in the expected screen and print states.
- Missing or malformed appearances are handled consistently.
- Unsupported annotation types are documented without silent visual loss.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run annotation visual comparisons.
- Run print-preview annotation checks.
- Run unsupported annotation snapshot tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
