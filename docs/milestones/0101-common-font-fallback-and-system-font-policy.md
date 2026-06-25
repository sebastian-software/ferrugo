# 0101: Common Font Fallback And System Font Policy

Status: done
Phase: 18
Size: medium
Depends on: 0100

## Goal

Make native rendering predictable when typical PDFs reference missing,
substituted, or system-resolved fonts.

## Scope

- Define deterministic font fallback order for native rendering.
- Add explicit handling for missing embedded font programs.
- Cache resolved fallback faces without leaking document-specific state.
- Add fixtures for office exports, invoices, and browser prints with missing
  fonts.

## Non-Goals

- Match every operating-system font resolver exactly.
- Download fonts at render time.
- Hide unsupported font classes behind silent substitution.

## Deliverables

- Font fallback policy and implementation notes.
- Fixture coverage for missing and substituted fonts.
- Native/PDFium comparison report with accepted drift.

## Acceptance Criteria

- Missing fonts produce stable output or a typed unsupported reason.
- Fallback resolution is deterministic across supported platforms.
- Cache size is bounded and observable in renderer metrics.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run font fallback corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added deterministic built-in fallback face and source classification for
  missing, substituted, standard-base, and embedded-program fonts.
- Added bounded fallback resolution caching and exposed
  `max_font_fallback_cache_entries` in native memory diagnostics.
- Added missing-font fixtures for browser-print, invoice/form, and
  office-export families.
- Added native render tests for the new fixtures and unit tests for fallback
  classification, cache bounding, and glyph bitmap face keys.
- Supported-family gate: 33 total, 33 native rendered, 0 fallback, 0 errors.
- PDFium visual comparison still marks the three missing-font fixtures as
  blockers; this milestone guarantees deterministic native output, not visual
  parity with PDFium font rasterization.
- Report: `docs/reports/font-fallback-policy-2026-06-25.md`.
- Implementation commit: `589eecd feat: add deterministic font fallback policy`.
