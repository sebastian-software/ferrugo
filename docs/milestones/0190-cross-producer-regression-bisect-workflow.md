# 0190: Cross-Producer Regression Bisect Workflow

Status: done
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

Completed on 2026-06-29.

- Added `producer-regression-report` to `pdfrust-cli`.
- The report groups native render outcomes by manifest producer tags,
  manifest family, and feature flags.
- Private or local-only fixture paths are redacted to local fixture IDs.
- Added milestone routing for common regression buckets such as optional
  content, image codecs, forms, annotations, color management, dense tables,
  and PDF 2.0.
- Added `docs/policies/producer-regression-bisect-workflow.md` as the local
  workflow for bisecting producer-scoped regressions without committing
  private artifacts.
- Recorded validation evidence in
  `docs/reports/cross-producer-regression-bisect-2026-06-29.md`.

The current producer compatibility manifest reports 15 manifest-mapped
fixtures: 13 native rendered, 2 typed fallback boundaries, and 0 errors. The
two fallback producer groups route to 0192 optional-content policy and 0209
native image codec deployment.
