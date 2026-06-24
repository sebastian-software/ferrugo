# 0027: Xref Streams And Object Streams

Status: done
Phase: 1
Size: medium
Depends on: 0026

## Goal

Load modern PDFs that use xref streams and object streams.

## Scope

- Parse xref stream dictionaries.
- Decode xref stream entries.
- Load compressed object streams.
- Resolve indirect objects from either classic xref tables or object streams.

## Non-Goals

- Full damaged-PDF repair.
- Hybrid-reference edge cases beyond the fixture set.
- Encryption.

## Deliverables

- Xref stream parser.
- Object stream loader.
- Fixtures covering browser-generated PDFs.

## Acceptance Criteria

- The document loader can enumerate pages from generated xref-stream fixtures.
- Object stream resolution is bounded and cycle-safe.
- Classic xref behavior remains covered by tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare page count against PDFium for selected fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Added `XrefStreamEntry`, `XrefStreamTable`, and `ModernDocument`.
- Added `load_modern_document` for PDFs whose `startxref` points at an `/XRef`
  stream object.
- Implemented `/W` and `/Index` xref stream entry decoding with checked
  big-endian field parsing.
- Added direct object loading for xref stream type-1 entries.
- Added object stream loading for type-2 compressed entries, with owned decoded
  `/ObjStm` buffers and on-demand object parsing through
  `ModernDocument::get_object`.
- Added validation that xref compressed-entry object IDs match the referenced
  object stream index.
- Added generated tests for a Flate-compressed xref stream, a Flate-compressed
  object stream containing a page dictionary, direct stream resolution, and an
  object-stream index mismatch.
- Hybrid-reference files and repair mode remain later milestones.
