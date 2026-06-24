# 0071: Transparency Stack Fidelity Gate

Status: todo
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

Empty until done.
