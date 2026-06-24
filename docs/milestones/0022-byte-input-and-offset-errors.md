# 0022: Byte Input And Offset Errors

Status: todo
Phase: 1
Size: small
Depends on: 0021

## Goal

Introduce a safe byte-input abstraction and parser error model with source
offsets.

## Scope

- Represent borrowed PDF bytes without unnecessary copying.
- Track byte offsets for diagnostics.
- Define syntax error variants for malformed input, unexpected EOF, invalid
  tokens, and unsupported constructs.
- Add tests for offset preservation and error formatting.

## Non-Goals

- Parse full PDF primitives.
- Recover damaged xref tables.
- Load files from disk inside the parser crate.

## Deliverables

- Byte input type.
- Parser result and error types.
- Unit tests for offset-aware failures.

## Acceptance Criteria

- Parser APIs accept borrowed bytes.
- Errors can report the byte offset that triggered failure.
- No heap allocation is required for normal byte scanning.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
