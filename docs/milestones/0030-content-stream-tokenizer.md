# 0030: Content Stream Tokenizer

Status: todo
Phase: 2
Size: medium
Depends on: 0029

## Goal

Tokenize page content streams into operands and operators.

## Scope

- Reuse primitive parsing for content-stream operands.
- Parse operator names.
- Preserve source offsets for operator errors.
- Add fixtures for malformed operator sequences.

## Non-Goals

- Execute graphics operators.
- Resolve resources.
- Render display lists.

## Deliverables

- Content token iterator.
- Operator token representation.
- Tests for valid and malformed content streams.

## Acceptance Criteria

- Simple generated page content can be tokenized end to end.
- Invalid tokens return typed errors with offsets.
- Tokenization avoids unnecessary allocation for common operators.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
