# 0173: Corrupt-But-Common PDF Recovery Corpus

Status: todo
Phase: 32
Size: medium
Depends on: 0172

## Goal

Handle common mildly corrupt PDFs predictably, either by recovering safely or by
returning precise typed errors.

## Scope

- Add fixtures for offset drift, duplicate objects, partial streams, malformed
  metadata, broken annotations, and recoverable page-tree issues.
- Define recovery budgets for parser and renderer paths.
- Implement bounded recovery for high-frequency benign corruption.
- Keep severe or ambiguous corruption as typed unsupported or parse errors.

## Non-Goals

- Accept arbitrary malformed input.
- Hide security-relevant corruption behind best-effort rendering.
- Add infinite search or repair loops.

## Deliverables

- Corrupt-but-common fixture corpus.
- Recovery policy updates.
- Parser and renderer diagnostics report.

## Acceptance Criteria

- Recoverable corrupt fixtures render deterministically.
- Non-recoverable fixtures fail with stable typed errors.
- Recovery paths have explicit time and memory budgets.

## Validation

- Run native-only `cargo test`.
- Run corrupt corpus classification.
- Run fuzz smoke tests for touched parser paths.
- Run benchmark subset for recovery cases.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
