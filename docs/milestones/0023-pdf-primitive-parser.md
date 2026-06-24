# 0023: PDF Primitive Parser

Status: todo
Phase: 1
Size: medium
Depends on: 0022

## Goal

Parse the core PDF primitive syntax needed by later object loading.

## Scope

- Parse null, booleans, integers, reals, names, strings, arrays, and
  dictionaries.
- Handle comments and whitespace according to PDF syntax rules.
- Preserve borrowed slices where safe and useful.
- Add generated fixture snippets for edge cases.

## Non-Goals

- Parse indirect objects.
- Decode streams.
- Interpret page content operators.

## Deliverables

- Primitive value enum.
- Parser functions for core PDF objects.
- Unit tests for valid and malformed primitive snippets.

## Acceptance Criteria

- The parser can round-trip or inspect primitive values from generated
  snippets.
- Malformed primitive syntax returns typed errors, not panics.
- String and name parsing behavior is documented where incomplete.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
