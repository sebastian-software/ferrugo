# 0069: Vertical And CJK Text Coverage

Status: todo
Phase: 11
Size: medium
Depends on: 0068

## Goal

Cover common CJK and vertical-writing PDFs well enough for native thumbnail
rendering.

## Scope

- Add vertical writing mode support where exposed by CMaps and font metrics.
- Test Japanese, Chinese, and Korean subset font documents.
- Handle multi-byte character codes without per-glyph heap churn.
- Document unsupported advanced typography cases.

## Non-Goals

- Full text extraction semantics.
- Complete layout shaping for source text.
- Locale-specific typography beyond PDF rendering instructions.

## Deliverables

- CJK and vertical text fixtures.
- Native renderer support for common vertical glyph positioning.
- Support matrix updates for international text documents.

## Acceptance Criteria

- Representative CJK PDFs render visible text natively.
- Vertical text placement is structurally correct for supported fixtures.
- Decode and glyph caches remain bounded on multi-page documents.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run CJK fixture comparisons against PDFium.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
