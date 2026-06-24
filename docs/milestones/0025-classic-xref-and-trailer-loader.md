# 0025: Classic Xref And Trailer Loader

Status: todo
Phase: 1
Size: medium
Depends on: 0024

## Goal

Load classic cross-reference tables and trailer dictionaries.

## Scope

- Locate `startxref`.
- Parse classic `xref` sections.
- Parse trailer dictionaries.
- Resolve reachable indirect objects through xref offsets.
- Report offset mismatches and malformed tables with typed errors.

## Non-Goals

- Parse xref streams.
- Repair severely damaged PDFs.
- Load encrypted PDFs.

## Deliverables

- Classic xref parser.
- Trailer loader.
- Document loader for simple generated PDFs.

## Acceptance Criteria

- The loader can enumerate objects from generated classic-xref fixtures.
- Offset and trailer errors include useful diagnostics.
- Malformed files fail without panics or out-of-bounds indexing.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare object count against PDFium for generated fixtures where practical.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
