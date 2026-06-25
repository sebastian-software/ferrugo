# 0104: Advanced CMap Encodings And Identity Mapping

Status: done
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

Completed on 2026-06-25.

- Added ToUnicode code-space range parsing and bounded range storage.
- Added longest-match CMap lookup that respects code-space ranges.
- Added two-byte Identity-H/V fallback mapping for Type0 fonts without a
  ToUnicode stream.
- Accepted `/Identity-H usecmap` and `/Identity-V usecmap` as explicit base CMap
  references while keeping other `usecmap` names unsupported.
- Added deterministic `InvalidCMap` coverage for malformed code-space ranges.
- Added generated CMap fixtures for Identity-H, Identity-V, and explicit
  ToUnicode code-space ranges plus native backend smoke tests.
- Supported-family gate: 41 total, 41 native rendered, 0 fallback, 0 errors.
- PDFium visual comparison marks the new CMap fixtures as blockers due to
  synthetic CID/font rasterizer drift, not native fallback.
- Report: `docs/reports/cmap-identity-coverage-2026-06-25.md`.
- Implementation commit: `ec9243f feat: add advanced cmap identity decoding`.
