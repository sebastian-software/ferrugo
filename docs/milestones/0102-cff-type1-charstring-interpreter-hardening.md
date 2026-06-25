# 0102: CFF Type1 Charstring Interpreter Hardening

Status: done
Phase: 18
Size: medium
Depends on: 0101

## Goal

Improve native glyph outline coverage for common CFF, Type1, and compact font
programs used in office and publishing PDFs.

## Scope

- Harden charstring parsing, stack limits, and subroutine recursion limits.
- Add coverage for common operators needed by generated business documents.
- Return typed font-program errors instead of panics or lossy fallbacks.
- Add targeted fixtures with CFF and Type1 embedded fonts.

## Non-Goals

- Implement a full font editor.
- Support unsafe unbounded recursion or arbitrary program execution.
- Claim typographic parity before visual comparison confirms it.

## Deliverables

- Hardened charstring interpreter path.
- Font-program error taxonomy updates.
- Differential report for CFF and Type1 fixtures.

## Acceptance Criteria

- Malformed charstrings fail safely with bounded memory use.
- Common CFF and Type1 glyph outlines render without PDFium fallback.
- Visual drift is measured and classified per corpus family.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run malformed font fixtures.
- Run font visual-diff comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added bounded Type1 charstring subset interpretation for synthetic FontFile
  programs.
- Added charstring stack and subroutine-depth limits to glyph outline options.
- Added typed glyph-outline errors for charstring stack overflow and
  subroutine overflow.
- Added CFF FontFile3 and Type1 FontFile generated fixtures plus native render
  smokes.
- Supported-family gate: 35 total, 35 native rendered, 0 fallback, 0 errors.
- PDFium visual comparison marks the new CFF/Type1 fixtures as blockers due to
  text rasterizer drift, not native fallback.
- Report: `docs/reports/cff-type1-charstring-hardening-2026-06-25.md`.
- Implementation commit: `97188f2 feat: harden type1 charstring outlines`.
