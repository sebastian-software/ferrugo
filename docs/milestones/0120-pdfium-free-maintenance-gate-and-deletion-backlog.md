# 0120: PDFium-Free Maintenance Gate And Deletion Backlog

Status: todo
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

Empty until done.
