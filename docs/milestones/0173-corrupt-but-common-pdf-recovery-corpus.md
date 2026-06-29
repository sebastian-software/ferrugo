# 0173: Corrupt-But-Common PDF Recovery Corpus

Status: done
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

Completed 2026-06-26.

- Added `fixtures/corrupt-recovery-manifest.tsv` and five new generated
  corrupt-but-common fixtures covering benign broken annotations, xref object
  mismatch, partial stream boundaries, missing page-tree structure, and
  malformed Info metadata.
- Kept recovery bounded: xref offset drift, malformed linearization hints,
  missing annotation references, and isolated metadata corruption are covered;
  xref mismatch, partial streams, and malformed page trees remain stable
  `malformed` errors.
- Updated `docs/policies/malformed-recovery.md` with the explicit accepted and
  non-recoverable cases.
- Recorded parser, renderer, benchmark, and fuzz-smoke evidence in
  `docs/reports/corrupt-common-recovery-corpus-2026-06-26.md`.

Validation:

- `cargo test -p ferrugo-native corrupt -- --nocapture`
- `cargo test -p ferrugo-native malformed_metadata -- --nocapture`
- `cargo test -p ferrugo-native xref_offset_drift -- --nocapture`
- Corrupt corpus recoverable gate, full classification, and benchmark subset.
- `cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke`
- `cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke`
- `cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke`
