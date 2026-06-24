# 0110: Overprint Simulation For Print-Oriented PDFs

Status: todo
Phase: 19
Size: medium
Depends on: 0109

## Goal

Provide a pragmatic native overprint simulation path for print-oriented PDFs
whose thumbnails should remain visually useful.

## Scope

- Interpret overprint graphics state flags.
- Simulate common overprint behavior in RGB output.
- Record approximation status in diagnostics.
- Add fixtures with separations, spot colors, and knockout interactions.

## Non-Goals

- Guarantee press-proof color accuracy.
- Implement full prepress validation.
- Make overprint simulation the default without measurable evidence.

## Deliverables

- Overprint simulation implementation or guarded policy fallback.
- Approximation diagnostics.
- Visual comparison report for print-oriented PDFs.

## Acceptance Criteria

- Typical overprint documents produce useful thumbnails or explicit fallback.
- Approximation status is visible to callers.
- Simulation does not regress non-overprint documents.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run overprint fixture comparisons.
- Run color regression corpus checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
