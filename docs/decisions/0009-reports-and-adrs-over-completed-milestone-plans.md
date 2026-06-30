# 0009: Reports And ADRs Over Completed Milestone Plans

Date: 2026-06-30.
Status: accepted.

## Context

The milestone sweep produced a large set of numbered planning documents. They
were useful while the project needed tightly sequenced execution and honest
completion tracking. After completion, the repository had three overlapping
sources of truth:

- milestone plans and completion notes;
- reports with evidence, commands, measurements, and readiness decisions;
- ADRs and policies with durable architecture and support decisions.

The milestone directory had become heavy enough to make the documentation map
harder to read. Most durable information already lives in reports, policies,
backlogs, architecture notes, and ADRs.

## Decision

Remove completed milestone planning documents from the active repository docs.

Going forward:

- ADRs record durable architecture and product decisions.
- Policies record operational rules, compatibility boundaries, support
  guarantees, and release requirements.
- Reports record evidence from gates, corpus sweeps, benchmarks, and readiness
  decisions.
- Backlogs record actionable follow-up work after a report or gate.
- Research notes record source-informed or ecosystem-informed findings.
- Short-lived milestone plans may still be created when useful, but completed
  plans should be retired once their decisions and evidence have durable homes.

Historical milestone content remains recoverable through Git history.

## Rationale

Milestone files are good execution scaffolding. They are poor permanent
documentation when every item is done, especially once each major slice already
has a report or policy.

Keeping the durable record in ADRs, policies, reports, and backlogs makes the
repo easier to navigate:

- readers find current truth faster;
- architecture decisions are separated from task checklists;
- completion evidence is preserved in reports;
- old execution mechanics do not compete with current product status.

## Consequences

Positive:

- The docs tree becomes smaller and easier to scan.
- Current decisions are easier to review.
- Future planning can be lighter and more targeted.
- Git history still preserves the full milestone sweep.

Tradeoffs:

- Historical report command lines may mention paths that no longer exist.
- Anyone investigating old execution order must use Git history or rollout
  summaries.
- Future work needs discipline to promote durable decisions before deleting
  planning artifacts.

## Follow-Up

- Update README, documentation guide, and roadmap to point at reports, ADRs,
  policies, and backlogs instead of `docs/milestones/`.
- Keep future completed planning artifacts out of the long-term navigation path
  unless they remain actively useful.
