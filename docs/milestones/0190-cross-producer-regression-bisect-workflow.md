# 0190: Cross-Producer Regression Bisect Workflow

Status: todo
Phase: 35
Size: medium
Depends on: 0189

## Goal

Make regressions actionable by grouping failures by producer, feature, and code
change so native renderer coverage can improve without manual guesswork.

## Scope

- Add report output that groups failures by producer metadata and feature flags.
- Document a local bisect workflow for renderer regressions.
- Add labels or categories that connect corpus failures to milestones.
- Keep private fixture paths and metadata out of committed artifacts.

## Non-Goals

- Build a hosted regression service.
- Commit private customer documents.
- Replace maintainer judgment for visual triage.

## Deliverables

- Regression bisect workflow documentation.
- Producer-grouped report output or issue template.
- Corpus governance updates for regression ownership.

## Acceptance Criteria

- Maintainers can identify affected producer families from a failed gate.
- Regression artifacts include enough detail to reproduce locally.
- Sensitive fixture details remain redacted or local-only.

## Validation

- Run report generation against the current corpus.
- Simulate a failed fixture classification.
- Review generated artifacts for privacy leaks.
- Run native-only `cargo test`.

## Completion Notes

Empty until done.
