# 0104: Advanced CMap Encodings And Identity Mapping

Status: todo
Phase: 18
Size: medium
Depends on: 0103

## Goal

Expand native text decoding for PDFs that use identity CMaps, custom embedded
CMaps, and non-trivial code-space ranges.

## Scope

- Parse multi-byte code-space ranges with bounded lookup tables.
- Support identity and embedded CMap mappings used by CID fonts.
- Add diagnostics for missing, malformed, or cyclic CMap resources.
- Cover CJK, vertical text, and subset font fixtures.

## Non-Goals

- Implement search indexing or text extraction quality scoring.
- Accept recursive CMap includes without limits.
- Treat visually correct glyph rendering as proof of Unicode extraction parity.

## Deliverables

- Advanced CMap parser and lookup path.
- CMap fixture set.
- Text decoding and visual comparison report.

## Acceptance Criteria

- Common CID font mappings render without PDFium fallback.
- CMap resource use has explicit size and recursion budgets.
- Malformed mappings return deterministic typed errors.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run CMap parser fixture tests.
- Run CJK and vertical text corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
