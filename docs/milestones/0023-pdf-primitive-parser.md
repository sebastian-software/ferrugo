# 0023: PDF Primitive Parser

Status: done
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

Completed on 2026-06-24.

- Added `PdfPrimitive`, `PdfNumber`, `PdfName`, and `PdfString` in
  `pdfrust-syntax`.
- Added `parse_primitive` for null, booleans, integers, reals, names, literal
  strings, hexadecimal strings, arrays, and dictionaries.
- Added whitespace and comment skipping for primitive parsing.
- Kept names and string contents borrowed from the original input; literal
  string escapes and hexadecimal bytes remain raw for later semantic decoding.
- Added tests for scalar values, borrowed names and strings, arrays,
  dictionaries, comments, trailing tokens, and malformed dictionaries.
