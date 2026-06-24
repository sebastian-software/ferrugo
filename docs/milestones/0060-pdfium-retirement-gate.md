# 0060: PDFium Retirement Gate

Status: todo
Phase: 8
Size: medium
Depends on: 0059

## Goal

Decide, from evidence, whether PDFium can move from primary renderer to
fallback, optional dependency, or removal candidate.

## Scope

- Run the full typical-document corpus through native and PDFium backends.
- Compare success rates, visual diffs, timeouts, memory use, and unsupported
  categories.
- Define a go/no-go threshold for making native rendering the default.
- Document remaining blockers and the next replacement phase if removal is not
  yet justified.

## Non-Goals

- Remove PDFium without measured evidence.
- Claim complete PDF specification coverage.
- Hide unsupported native cases behind silent degradation.

## Deliverables

- PDFium retirement decision report.
- Native-default or fallback-only rollout recommendation.
- Follow-up milestones for any remaining blockers.

## Acceptance Criteria

- The project has a concrete decision on PDFium's role after this phase.
- Native renderer coverage is measured against real corpus categories.
- Remaining gaps are explicit enough to plan the next agile slice.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run the full local corpus comparison suite.
- Run representative performance and memory measurements.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
