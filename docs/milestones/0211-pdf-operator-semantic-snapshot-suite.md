# 0211: PDF Operator Semantic Snapshot Suite

Status: todo
Phase: 40
Size: medium
Depends on: 0210

## Goal

Add semantic snapshots for PDF graphics and text operators so the Rust-native
renderer can detect behavior drift before it becomes visual regressions in
typical documents.

## Scope

- Generate reduced operator fixtures for graphics state, paths, text state,
  images, form XObjects, transparency, patterns, and annotations.
- Snapshot normalized display-list or render-trace semantics for supported
  operators.
- Attach unsupported and partial operator states to typed diagnostics.
- Keep snapshots stable across platforms by avoiding pixel-only comparisons for
  semantic behavior.

## Non-Goals

- Replace visual corpus testing.
- Snapshot internal implementation details that should remain refactorable.
- Cover every PDF operator before it appears in corpus evidence.

## Deliverables

- Operator semantic snapshot suite.
- Operator state matrix update.
- Drift triage report for existing renderer behavior.

## Acceptance Criteria

- Common operators have stable semantic snapshots.
- Renderer refactors can detect high-impact operator drift early.
- Unsupported operators remain typed and visible in diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run semantic snapshot tests.
- Run operator coverage scan.
- Run reduced fixture visual smoke checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
