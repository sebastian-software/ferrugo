# 0026: Streams And Basic Filters

Status: todo
Phase: 1
Size: medium
Depends on: 0025

## Goal

Read PDF streams and decode the first common lossless filters.

## Scope

- Parse stream dictionaries and raw stream byte ranges.
- Implement `FlateDecode`, `ASCIIHexDecode`, and `ASCII85Decode`.
- Support filter arrays with bounded expansion.
- Add decompressed-size limits for safety.

## Non-Goals

- Decode JPEG, JPEG 2000, CCITT, or JBIG2.
- Interpret content streams.
- Implement predictor filters unless needed by the generated fixtures.

## Deliverables

- Stream object representation.
- Basic filter pipeline.
- Tests for valid streams, malformed filters, and expansion limits.

## Acceptance Criteria

- Generated compressed content streams can be decoded.
- Unsupported filters return the stable unsupported class at the facade
  boundary.
- Expansion limits prevent unbounded memory growth.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
