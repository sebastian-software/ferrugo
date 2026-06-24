# 0030: Content Stream Tokenizer

Status: done
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

Completed with the `feat: tokenize content streams` change.

- Added `pdfrust-content` tokenizer APIs:
  `tokenize_content`, `ContentTokenizer`, `ContentToken`, `OperatorName`,
  `ContentError`, and `ContentErrorKind`.
- Reused `pdfrust-syntax::parse_primitive_prefix` for operands while keeping
  operator names borrowed and allocation-free.
- Preserved absolute source offsets for operands, operators, syntax failures,
  comments, and invalid delimiter-started operators.
- Added tests for simple graphics/text operators, comments, boolean/null
  keyword disambiguation, malformed operands, invalid operator delimiters, and
  end-to-end tokenization of `fixtures/generated/text-page.pdf`.
- Validation:
  - `cargo fmt --check`
  - `cargo test -p pdfrust-content`
