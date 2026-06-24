# 0025: Classic Xref And Trailer Loader

Status: done
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

Completed on 2026-06-24.

- Added `ClassicXrefEntry`, `ClassicXrefTable`, `Trailer`, and
  `ClassicDocument`.
- Added `load_classic_document` for simple PDFs with classic xref tables.
- Added `startxref` location, xref subsection parsing, trailer dictionary
  parsing, and in-use object loading through xref offsets.
- Added offset mismatch diagnostics when an xref entry points at a different
  object ID.
- Extended the primitive parser with `PdfPrimitive::Reference` so trailer
  dictionaries can represent `/Root 1 0 R`.
- Added generated in-test classic PDF fixtures for successful load, xref offset
  mismatch, and missing `startxref`.
