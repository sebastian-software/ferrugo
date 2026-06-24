# 0093: Incremental Update Edge Case Hardening

Status: todo
Phase: 16
Size: medium
Depends on: 0092

## Goal

Harden parser behavior for real-world incremental updates and hybrid reference
structures that remain common in edited PDFs.

## Scope

- Expand tests for multiple revisions, deleted objects, hybrid xrefs, and
  object replacement.
- Ensure page tree and resource lookup uses the effective latest revision.
- Keep malformed revision chains bounded by explicit limits.
- Add diagnostics that identify the failing revision or reference section.

## Non-Goals

- Write incremental updates.
- Recover every damaged file at the cost of ambiguous behavior.
- Keep all historical revisions resident after resolution.

## Deliverables

- Incremental update edge-case coverage.
- Parser hardening fixes.
- Error diagnostics for revision failures.

## Acceptance Criteria

- Common edited PDFs render from the latest effective object graph.
- Corrupt update chains fail with bounded, useful errors.
- Object resolution avoids unnecessary historical object retention.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run incremental-update corpus comparisons.
- Run parser memory checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
