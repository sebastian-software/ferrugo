# 0019: Timeout And Isolation Decision

Status: todo
Phase: 1
Size: small
Depends on: 0018

## Goal

Decide how Phase 1 enforces render timeouts and contains hostile PDFs.

## Scope

- Evaluate serialized in-process rendering.
- Evaluate worker-thread timeout boundaries.
- Evaluate process isolation for hard cancellation.
- Record the chosen next implementation path.

## Non-Goals

- Implement a production sandbox.
- Add batch scheduling.
- Add Node-API timeout behavior.

## Deliverables

- Decision document under `docs/decisions/`.
- Follow-up implementation milestones if needed.

## Acceptance Criteria

- Timeout semantics are explicit for CLI/library callers.
- Security and memory tradeoffs are documented.
- The next implementation slice is small enough to validate independently.

## Validation

- Cross-check the decision against `docs/errors.md` and the Phase 0 report.

## Completion Notes

Empty until done.
