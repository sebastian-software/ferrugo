# 0071: Transparency Stack Fidelity Gate

Status: done
Phase: 11
Size: medium
Depends on: 0070

## Goal

Raise transparency fidelity enough for common modern documents to avoid PDFium
fallback.

## Scope

- Revisit soft masks, transparency groups, isolated groups, and blend modes with
  corpus evidence.
- Add stack accounting for nested transparency operations.
- Optimize intermediate buffers for thumbnail-sized rendering.
- Document unsupported print-production cases.

## Non-Goals

- Full prepress-grade compositing.
- Device-specific overprint simulation beyond existing policy.
- Unlimited nested transparency buffers.

## Deliverables

- Transparency fidelity tests and corpus report.
- Bounded transparency stack implementation improvements.
- Support matrix updates for transparency-heavy documents.

## Acceptance Criteria

- Common transparency-heavy PDFs render without missing major visual layers.
- Nested transparency obeys explicit buffer and recursion budgets.
- Remaining fallback cases are measurable and categorized.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run transparency corpus comparisons.
- Run memory checks for nested group fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `feat: composite extgstate alpha paths` implementation and
the `docs: complete transparency stack gate` report update.

- Added ExtGState `/ca` and `/CA` decoding and independent fill/stroke alpha in
  the native display-list graphics state.
- Applied alpha-aware compositing to path fills, tiling-pattern fills, and
  strokes while preserving existing blend-mode behavior.
- Added `fixtures/generated/transparency-alpha.pdf`, manifest/docs metadata,
  unit tests, and a native backend fixture test.
- Recorded validation and remaining limitations in
  `docs/reports/transparency-stack-coverage-2026-06-24.md`.
