# 0061: Native Renderer Default Rollout

Status: todo
Phase: 9
Size: medium
Depends on: 0060

## Goal

Make the Rust renderer the default for the document categories that passed the
PDFium retirement gate, while keeping fallback behavior explicit.

## Scope

- Add backend selection rules for native-first rendering.
- Gate native defaulting behind support-matrix categories and feature flags.
- Preserve PDFium fallback for known unsupported or high-risk documents.
- Document how callers can force native, force PDFium, or use automatic mode.

## Non-Goals

- Remove PDFium binaries or bindings.
- Claim native coverage for categories not validated in 0060.
- Silently fall back without diagnostics.

## Deliverables

- Native-first backend selection policy.
- CLI and library documentation for backend modes.
- Regression tests for supported, fallback, and forced-backend paths.

## Acceptance Criteria

- Supported fixtures render through native by default.
- Unsupported fixtures either fall back or report a documented error.
- Backend choice is visible in diagnostics and comparison output.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run supported and fallback fixture comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
