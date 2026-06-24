# 0057: Malformed PDF Recovery Budget

Status: done
Phase: 7
Size: medium
Depends on: 0056

## Goal

Recover from common malformed PDF structure only where it improves real corpus
coverage without hiding data-corruption risks.

## Scope

- Identify malformed cases that PDFium renders and the native backend rejects.
- Implement bounded recovery for missing whitespace, offset drift, and trailer
  discovery where safe.
- Add recovery budgets for scans, recursion, and allocations.
- Emit diagnostics when recovery changes normal parsing behavior.

## Non-Goals

- Best-effort rendering of arbitrary damaged files.
- Silent repair of corrupt security-sensitive structures.
- Unbounded whole-file rescans.

## Deliverables

- Recovery policy document.
- Parser recovery hooks with budgets.
- Malformed fixture tests.

## Acceptance Criteria

- Selected malformed fixtures render or fail with better diagnostics.
- Recovery cannot turn into unbounded scanning.
- Strict parsing behavior remains testable.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run malformed corpus comparisons against PDFium.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `docs/policies/malformed-recovery.md` with explicit supported and
  non-recoverable recovery cases.
- Added bounded xref object-offset drift recovery using
  `DEFAULT_XREF_OFFSET_RECOVERY_SCAN_BYTES`; strict parsing is tried first and
  unrecovered failures preserve the original error.
- Added generated `fixtures/generated/malformed-xref-offset-drift.pdf` plus
  object-loader and native-render regression coverage.
