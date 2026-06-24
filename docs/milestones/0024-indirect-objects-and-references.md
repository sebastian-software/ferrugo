# 0024: Indirect Objects And References

Status: done
Phase: 1
Size: medium
Depends on: 0023

## Goal

Load indirect PDF objects and references into a safe object model.

## Scope

- Parse object headers such as `12 0 obj`.
- Represent object numbers, generation numbers, and references with typed
  wrappers.
- Store loaded objects in an object table.
- Detect duplicate or malformed object definitions.

## Non-Goals

- Resolve xref tables.
- Decode object streams.
- Interpret page dictionaries.

## Deliverables

- Typed object IDs and references.
- Object table data structure.
- Unit tests for indirect object parsing and lookup.

## Acceptance Criteria

- Borrowed input can be parsed into owned object metadata.
- Invalid object IDs and duplicate entries return typed errors.
- Object table lookups do not expose unchecked indexes.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Added typed `ObjectNumber`, `GenerationNumber`, `ObjectId`, and `Reference`.
- Added `IndirectObject<'a>` and `ObjectTable<'a>` with duplicate detection.
- Added `parse_reference` for `12 0 R` style references.
- Added `parse_indirect_object` for contiguous `obj ... endobj` slices.
- Added typed `ObjectError` diagnostics with offsets where available.
- Added tests for references, invalid IDs, indirect object parsing, missing
  `endobj`, lookup, and duplicate rejection.
