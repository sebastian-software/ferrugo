# 0026: Streams And Basic Filters

Status: done
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

Completed on 2026-06-24.

- Added `ObjectValue::Stream` and `StreamObject` so indirect stream objects
  keep dictionary metadata, borrowed raw stream bytes, and raw byte offsets.
- Added `parse_primitive_prefix` for parser layers that need one primitive
  followed by additional structure such as `stream`.
- Added bounded stream decoding through `StreamDecodeOptions`, including
  `FlateDecode`, `ASCIIHexDecode`, and `ASCII85Decode`.
- Added filter-array handling with aliases `/Fl`, `/AHx`, and `/A85`.
- Added typed errors for unsupported filters, unsupported stream lengths,
  decode failures, and decoded-size limit violations.
- Added tests for raw stream ranges, generated compressed content streams,
  filter arrays, malformed filter data, unsupported filters, and expansion
  limits.
- Direct `/Length` integers are supported. Indirect stream lengths remain a
  later object-resolution task.
