# 0069: Vertical And CJK Text Coverage

Status: done
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

Completed on 2026-06-24.

- Added `TextWritingMode` metadata to font descriptors.
- Accepted `Identity-V` Type0/CID font encoding and advanced text along the
  negative text Y axis for vertical writing.
- Added render tests for `Identity-V` ToUnicode decoding and vertical glyph
  origins.
- Added `fixtures/generated/vertical-cjk-text.pdf`, a generated Type0/CID
  Japanese vertical text fixture.
- Added native backend coverage for rendering the generated vertical CJK fixture
  without fallback.
- Recorded fallback and PDFium comparison results in
  `docs/reports/vertical-cjk-coverage-2026-06-24.md`.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo test -p pdfrust-render`
  - `cargo test -p pdfrust-native`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - CJK fallback summary through the office-export family
  - native/PDFium render comparison for `vertical-cjk-text.pdf`
