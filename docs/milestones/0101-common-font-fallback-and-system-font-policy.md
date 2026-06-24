# 0101: Common Font Fallback And System Font Policy

Status: todo
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

Empty until done.
