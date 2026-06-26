# 0164: Independent Reference Oracle Strategy

Status: done
Phase: 30
Size: medium
Depends on: 0163

## Goal

Define a PDFium-free reference strategy for future visual validation so the
renderer can keep improving without relying on PDFium as the primary oracle.

## Scope

- Compare candidate reference sources such as committed golden images,
  producer-generated expected output, independent renderers, and manual review.
- Define when historical PDFium comparisons may still be used as archived
  evidence.
- Add a plan for calibrating thresholds without masking native regressions.
- Document which validation paths are suitable for CI, maintainer review, and
  release gates.

## Non-Goals

- Delete existing historical PDFium reports.
- Build a full replacement dashboard in this milestone.
- Accept a single opaque renderer as an unquestioned oracle.

## Deliverables

- Reference oracle strategy document.
- Validation mode taxonomy.
- Follow-up backlog for golden image and review tooling.

## Acceptance Criteria

- Release validation has a PDFium-free path.
- Oracle choices are tied to document families and risk.
- Ambiguous cases have a documented manual-review fallback.

## Validation

- Review existing visual diff reports and corpus family needs.
- Run native-only corpus gate.
- Run a sample golden-image comparison if tooling exists.
- Verify docs distinguish runtime, comparison, and historical evidence.

## Completion Notes

Completed on 2026-06-26.

- Added `docs/policies/reference-oracle-strategy.md` with the validation mode
  taxonomy, document-family routing, threshold calibration rules, and manual
  review fallback.
- Added `docs/backlogs/reference-oracle-tooling-backlog.md` for golden image,
  multi-oracle, and manual-review tooling.
- Added
  `docs/reports/independent-reference-oracle-strategy-2026-06-26.md`.
- Clarified `docs/policies/visual-diff-thresholds.md` and `docs/baselines.md`
  so PDFium visual reports are maintainer comparison evidence, not runtime or
  release-gate evidence.
