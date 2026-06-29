# 0216: Cross-Producer Typical Document Fusion Corpus

Status: done
Phase: 41
Size: medium
Depends on: 0215

## Goal

Build a fused corpus that combines office suites, browsers, scanners, mobile
apps, report generators, government systems, and design tools into a single
typical-document confidence gate.

## Scope

- Classify equivalent document workflows across multiple producers.
- Add reduced fixtures that isolate producer-specific differences for the same
  document family.
- Track renderer behavior by workflow, producer, feature category, and profile.
- Keep privacy review and fixture minimization requirements enforced.

## Non-Goals

- Store private user documents.
- Prefer producer popularity over measured user workflow impact.
- Remove focused feature corpora that still catch specific regressions.

## Deliverables

- Cross-producer typical-document corpus.
- Producer compatibility matrix update.
- Fixture minimization and privacy report.

## Acceptance Criteria

- Common workflows have coverage across multiple producer implementations.
- Producer-specific failures are linked to reduced fixtures and typed causes.
- The corpus remains runnable in native-only CI profiles.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run cross-producer corpus gate.
- Run fixture privacy and minimization checks.
- Run producer compatibility matrix generation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Added `fixtures/cross-producer-fusion-manifest.tsv` with 22 generated rows
  across report export, tabular statement, form-with-marks, scan ingest,
  dashboard/map export, and explicit unsupported-boundary workflows.
- Added `fixtures/cross-producer-fusion-matrix.tsv` with workflow, producer,
  profile, expected status, typed cause, owner route, and minimization notes.
- Added `scripts/check_cross_producer_fusion_corpus.sh` to validate manifest
  shape, matrix shape, generated-source provenance, expected tags, producer and
  workflow coverage, privacy tags, owner routes, and minimization notes.
- Documented the corpus in `docs/corpus-taxonomy.md` and linked the fusion
  manifest into the producer regression workflow.
- Report: `docs/reports/cross-producer-fusion-corpus-2026-06-29.md`.

Validation:

- `bash scripts/check_cross_producer_fusion_corpus.sh`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --fail-on-fallback --max-edge 160 --output target/cross-producer-fusion-0216-supported-gate.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-unsupported-boundary --max-edge 160 --output target/cross-producer-fusion-0216-boundary-classification.json`
- `cargo run -p ferrugo-cli --no-default-features -- producer-regression-report fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --max-edge 160 --output target/cross-producer-fusion-0216-producer-report.json`
- `cargo fmt --check`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `git diff --check -- fixtures/cross-producer-fusion-manifest.tsv fixtures/cross-producer-fusion-matrix.tsv scripts/check_cross_producer_fusion_corpus.sh docs/corpus-taxonomy.md docs/policies/producer-regression-bisect-workflow.md docs/milestones/README.md docs/milestones/0216-cross-producer-typical-document-fusion-corpus.md docs/reports/cross-producer-fusion-corpus-2026-06-29.md`
