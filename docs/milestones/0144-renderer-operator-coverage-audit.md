# 0144: Renderer Operator Coverage Audit

Status: todo
Phase: 26
Size: medium
Depends on: 0143

## Goal

Audit PDF graphics and text operator coverage against the corpus so missing
native renderer behavior is visible before deeper fidelity work begins.

## Scope

- Record operator usage across supported and near-supported corpus files.
- Map operators to implemented, partial, unsupported, and ignored states.
- Identify high-impact missing operators for typical documents.
- Add typed fallback reasons where unsupported operators are still ambiguous.

## Non-Goals

- Implement large new operator families in this audit.
- Track interactive viewer behavior.
- Treat rare adversarial operators as equal priority to common documents.

## Deliverables

- Operator coverage matrix.
- Unsupported-operator fallback taxonomy updates.
- Prioritized implementation candidates.

## Acceptance Criteria

- Common corpus operators have explicit native support status.
- Unsupported operators produce typed reasons rather than silent drift.
- High-impact gaps are tied to specific follow-up milestones.

## Validation

- Run corpus operator scan.
- Run native-only `cargo test`.
- Run fallback reason snapshot tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
