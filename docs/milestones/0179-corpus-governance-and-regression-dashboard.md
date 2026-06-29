# 0179: Corpus Governance And Regression Dashboard

Status: done
Phase: 33
Size: medium
Depends on: 0178

## Goal

Make corpus growth sustainable by tracking fixture provenance, coverage, visual
status, performance, memory, and regression ownership in one maintainer flow.

## Scope

- Define required metadata for generated, public, private, and local-only
  fixtures.
- Add dashboard or report output for coverage, failures, budget violations, and
  unsupported categories.
- Document review rules for adding or removing fixtures.
- Connect regression entries to milestone follow-ups.

## Non-Goals

- Commit private or unlicensed PDFs.
- Replace every local maintainer workflow with a web service.
- Hide unsupported cases to improve headline coverage.

## Deliverables

- Corpus governance policy.
- Regression dashboard or generated report.
- Fixture metadata validation updates.

## Acceptance Criteria

- Corpus entries have provenance and license handling.
- Regressions are visible with owner, category, and severity.
- Dashboard output supports native-only release decisions.

## Validation

- Run fixture metadata validation.
- Run dashboard/report generation.
- Run native-only corpus gate.
- Review regression categories for stale or ambiguous entries.

## Completion Notes

Completed on 2026-06-26.

Added a native-only corpus dashboard maintainer flow:

- `scripts/generate_corpus_dashboard.sh` generates metadata, local corpus
  validation, support classification, operator coverage, performance, batch, and
  compact dashboard JSON artifacts under `target/`.
- `docs/policies/corpus-governance.md` defines required manifest metadata,
  license/provenance handling, fixture review rules, regression visibility, and
  privacy boundaries.
- `docs/reports/corpus-governance-dashboard-2026-06-26.md` records the first
  dashboard run and release-decision summary.

Validation completed:

- `bash scripts/generate_corpus_dashboard.sh target/corpus-dashboard-0179`
- `cargo run -p ferrugo-cli --no-default-features -- validate-local-corpus fixtures/local-corpus.example.toml --allow-missing`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
