# 0055: Incremental Updates And Hybrid References

Status: in-progress
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

In progress:

- First implementation slice follows classic trailer `/Prev` chains with a
  `16`-revision depth limit and cycle detection.
- Classic xref entries are merged newest-first so later reachable object
  revisions win while older xrefs still fill missing objects.
- Added object-loader tests for latest object revision selection,
  incremental-update cycles, and incremental-update depth overflow.
- Current validation:
  - `cargo test -p pdfrust-object incremental -- --nocapture`
