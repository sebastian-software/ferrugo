# Milestones

This directory tracks small, stable project milestones. Milestones are intended
to feel completable: most should fit into half a day to two days of focused
work.

Do not move milestone files between `todo` and `done` directories. Keep paths
stable and update the status in both places:

1. The `Status:` field in the milestone document.
2. The table in this index.

Allowed statuses:

- `todo`
- `in-progress`
- `done`
- `blocked`

## Todo

| ID | Milestone | Phase | Size | Depends On |
| --- | --- | --- | --- | --- |
| 0008 | [Fixture Policy And Seed Fixtures](0008-fixture-policy-and-seed-fixtures.md) | 0 | small | 0003 |
| 0009 | [Rust Workspace Skeleton](0009-rust-workspace-skeleton.md) | 0 | small | 0003 |
| 0010 | [Thumbnail API Facade](0010-thumbnail-api-facade.md) | 0 | small | 0009 |
| 0011 | [PDFium Backend Linkage](0011-pdfium-backend-linkage.md) | 0 | medium | 0006, 0010 |
| 0012 | [Render Page Zero To RGBA](0012-render-page-zero-to-rgba.md) | 0 | small | 0011 |
| 0013 | [PNG Output CLI](0013-png-output-cli.md) | 0 | small | 0012 |
| 0014 | [Error Taxonomy Mapping](0014-error-taxonomy-mapping.md) | 0 | small | 0011 |
| 0015 | [Differential Baseline Format](0015-differential-baseline-format.md) | 0 | small | 0012, 0014 |
| 0016 | [Phase 0 Report And Pivot Decision](0016-phase-0-report-and-pivot-decision.md) | 0 | small | 0007, 0015 |

## In Progress

No milestones are currently in progress.

## Done

| ID | Milestone | Phase | Size | Completed |
| --- | --- | --- | --- | --- |
| 0001 | [Milestone Tracking Structure](0001-milestone-tracking-structure.md) | 0 | small | 2026-06-24 |
| 0002 | [Research And Porting Baseline](0002-research-and-porting-baseline.md) | 0 | small | 2026-06-24 |
| 0003 | [Phase 0 Decision Baseline](0003-phase-0-decision-baseline.md) | 0 | small | 2026-06-24 |
| 0004 | [License Files And Attribution Policy](0004-license-files-and-attribution-policy.md) | 0 | small | 2026-06-24 |
| 0005 | [PDFium Source Checkout Recipe](0005-pdfium-source-checkout-recipe.md) | 0 | small | 2026-06-24 |
| 0006 | [Minimal PDFium GN Configuration](0006-minimal-pdfium-gn-configuration.md) | 0 | small | 2026-06-24 |
| 0007 | [PDFium Build Measurement Baseline](0007-pdfium-build-measurement-baseline.md) | 0 | medium | 2026-06-24 |

## Update Rules

- When starting work, move the row from `Todo` to `In Progress` and set
  `Status: in-progress`.
- When completing work, move the row to `Done`, set `Status: done`, and fill in
  `Completion Notes` with commits, measurements, artifacts, and follow-ups.
- When blocked, move the row to a `Blocked` section if needed, set
  `Status: blocked`, and document the unblock condition.
- Keep milestones small. If a milestone grows beyond two focused days, split it.
