# 0199: Unsupported Feature Burn-Down Release Candidate Gate

Status: todo
Phase: 37
Size: medium
Depends on: 0198

## Goal

Turn the remaining unsupported-feature backlog into a 1.2 release-candidate
decision with explicit burn-down, deferral, and release-blocking categories.

## Scope

- Re-run unsupported classification across the expanded corpus.
- Separate release blockers from documented unsupported boundaries.
- Confirm that high-frequency typical-document gaps have owner milestones or
  accepted deferrals.
- Produce the final 1.2 readiness checklist.

## Non-Goals

- Implement every remaining unsupported feature.
- Hide unsupported outcomes from consumer APIs.
- Defer release blockers without a documented decision.

## Deliverables

- Unsupported feature burn-down report.
- Release-blocker and deferral list.
- Updated 1.2 readiness checklist.

## Acceptance Criteria

- Every frequent unsupported feature has a decision.
- Release blockers are measurable and reproducible.
- Accepted deferrals are documented in public support boundaries.

## Validation

- Run native-only `cargo test`.
- Run full unsupported classification.
- Run supported corpus gate.
- Review support matrix and public docs.

## Completion Notes

Empty until done.
