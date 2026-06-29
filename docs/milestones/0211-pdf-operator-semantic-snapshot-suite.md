# 0211: PDF Operator Semantic Snapshot Suite

Status: done
Phase: 40
Size: medium
Depends on: 0210

## Goal

Add semantic snapshots for PDF graphics and text operators so the Rust-native
renderer can detect behavior drift before it becomes visual regressions in
typical documents.

## Scope

- Generate reduced operator fixtures for graphics state, paths, text state,
  images, form XObjects, transparency, patterns, and annotations.
- Snapshot normalized display-list or render-trace semantics for supported
  operators.
- Attach unsupported and partial operator states to typed diagnostics.
- Keep snapshots stable across platforms by avoiding pixel-only comparisons for
  semantic behavior.

## Non-Goals

- Replace visual corpus testing.
- Snapshot internal implementation details that should remain refactorable.
- Cover every PDF operator before it appears in corpus evidence.

## Deliverables

- Operator semantic snapshot suite.
- Operator state matrix update.
- Drift triage report for existing renderer behavior.

## Acceptance Criteria

- Common operators have stable semantic snapshots.
- Renderer refactors can detect high-impact operator drift early.
- Unsupported operators remain typed and visible in diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run semantic snapshot tests.
- Run operator coverage scan.
- Run reduced fixture visual smoke checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `fixtures/operator-semantic-snapshot-manifest.tsv` for reduced text,
  path, inline-image, and pattern operator snapshots.
- Added `operator_coverage_should_match_semantic_snapshots` to freeze operator
  counts, support status, and typed fallback buckets for the selected fixtures.
- Documented the new manifest in `docs/corpus-taxonomy.md`.
- Produced `docs/reports/pdf-operator-semantic-snapshot-suite-2026-06-29.md`.

Validation run:

- `cargo fmt --check`
- `cargo test -p pdfrust-native operator_coverage_should_match_semantic_snapshots -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family text-state --include-family path-state --include-family image-state --include-family pattern-state --output target/operator-snapshot-0211-operators.json`
- `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family text-state --include-family path-state --include-family image-state --include-family pattern-state --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/operator-snapshot-0211-poppler.json`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
