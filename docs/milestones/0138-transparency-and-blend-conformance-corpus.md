# 0138: Transparency And Blend Conformance Corpus

Status: todo
Phase: 25
Size: medium
Depends on: 0137

## Goal

Create a focused conformance corpus for transparency groups, soft masks, alpha
constants, and blend modes used by typical generated PDFs.

## Scope

- Add fixtures for isolated groups, knockout boundaries, luminosity masks, and
  common blend modes.
- Compare native output against PDFium with documented thumbnail tolerances.
- Classify hard blockers separately from accepted anti-aliasing drift.
- Profile intermediate surface allocations and compositing cost.

## Non-Goals

- Claim full print-proof transparency conformance.
- Implement unsupported blend modes without evidence of common need.
- Ignore memory budgets for nested groups.

## Deliverables

- Transparency conformance fixture family.
- Blend and mask support matrix update.
- Compositing benchmark and memory report.

## Acceptance Criteria

- Common transparency cases render natively or fail with typed reasons.
- Nested groups stay inside intermediate-surface budgets.
- Visual-diff policy distinguishes tolerable drift from compositing errors.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run transparency conformance visual comparisons.
- Run compositing memory benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
