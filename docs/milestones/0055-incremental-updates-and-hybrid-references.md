# 0055: Incremental Updates And Hybrid References

Status: todo
Phase: 7
Size: medium
Depends on: 0054

## Goal

Load PDFs that use incremental updates, hybrid-reference files, or multiple
trailers as common producer output.

## Scope

- Follow `Prev` trailer chains with cycle and depth limits.
- Merge object revisions according to latest reachable xref data.
- Support hybrid-reference files when both classic xref and xref streams are
  present.
- Add fixtures for edited, signed, and saved-as PDFs.

## Non-Goals

- Signature validation.
- Repairing arbitrary corrupt update chains.
- Writing incremental updates.

## Deliverables

- Incremental xref resolver.
- Revision merge tests.
- Fixtures for multi-revision PDFs.

## Acceptance Criteria

- Latest object revisions are used for supported incremental files.
- Cyclic or oversized revision chains fail with typed errors.
- Hybrid-reference behavior is documented and covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run corpus comparisons for edited and signed PDFs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
