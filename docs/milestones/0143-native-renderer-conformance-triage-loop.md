# 0143: Native Renderer Conformance Triage Loop

Status: todo
Phase: 26
Size: medium
Depends on: 0142

## Goal

Turn visual and structural renderer differences into a repeatable triage loop
that produces small actionable Rust-native follow-up milestones.

## Scope

- Classify remaining differences by renderer subsystem and document family.
- Separate expected drift, unsupported PDF features, bugs, and test weaknesses.
- Add a stable report format for conformance triage.
- Produce a prioritized backlog with owner-ready slices.

## Non-Goals

- Fix every classified issue in this milestone.
- Treat PDFium as the only valid output for ambiguous rendering cases.
- Hide family-level failures behind aggregate scores.

## Deliverables

- Conformance triage report.
- Updated support matrix with subsystem tags.
- Follow-up backlog for renderer fidelity work.

## Acceptance Criteria

- Each blocker has a subsystem, fixture family, and recommended next action.
- Expected drift is explicitly justified.
- New work items are small enough for isolated implementation commits.

## Validation

- Run full supported corpus visual comparison.
- Run native-only supported corpus gate.
- Run report schema validation if available.
- Spot-check representative artifacts for each blocker class.

## Completion Notes

Empty until done.
