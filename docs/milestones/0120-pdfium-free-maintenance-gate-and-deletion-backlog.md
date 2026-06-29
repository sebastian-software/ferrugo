# 0120: PDFium-Free Maintenance Gate And Deletion Backlog

Status: done
Phase: 21
Size: medium
Depends on: 0119

## Goal

Decide which PDFium comparison, fallback, and packaging paths can be deleted,
kept as maintainer-only tooling, or retained for unsupported categories.

## Scope

- Audit all PDFium-linked code paths after the native renderer coverage gates.
- Split deletion candidates from maintainer-only comparison infrastructure.
- Verify native-only package, CLI, and library behavior.
- Produce a deletion backlog with risk and rollback notes.

## Non-Goals

- Delete comparison tooling without replacement evidence.
- Claim full PDF specification coverage.
- Remove documented unsupported-category escape hatches prematurely.

## Deliverables

- PDFium-free maintenance gate report.
- Deletion and retention backlog.
- Native-only package validation evidence.

## Acceptance Criteria

- Normal supported-document rendering works without PDFium installed.
- Remaining PDFium references are explicitly justified or scheduled for removal.
- Deletion work is split into small, reversible commits.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run native-only package validation.
- Run supported corpus gate.
- Run PDFium-enabled comparison tooling only as maintainer evidence.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Confirmed `cargo tree -p ferrugo-cli --no-default-features` has no
  `ferrugo-pdfium` dependency edge.
- Confirmed `cargo tree -p ferrugo-cli --features pdfium` adds
  `ferrugo-pdfium` only through the explicit feature.
- Added `docs/backlogs/pdfium-free-maintenance-backlog.md` with keep, delete,
  and deferred-deletion decisions plus rollback notes.
- Updated `docs/packaging.md` with the 0120 native-only maintenance gate and
  package validation result.
- Native supported-family gate passed with 46/46 native renders, 0 fallbacks,
  and 0 errors.
- CLI package dry-run is blocked by unpublished internal crates, not by PDFium;
  `ferrugo-syntax` and `ferrugo-thumbnail` leaf package dry-runs pass.
- PDFium-enabled benchmark remains maintainer evidence only.
- Report: `docs/reports/pdfium-free-maintenance-gate-2026-06-25.md`.
