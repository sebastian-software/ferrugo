# 0027: Xref Streams And Object Streams

Status: todo
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

Empty until done.
