# 0199: Unsupported Feature Burn-Down Release Candidate Gate

Status: done
Phase: 37
Size: medium
Depends on: 0198

## Goal

Turn the remaining unsupported-feature backlog into a 1.2 release-candidate
decision with explicit burn-down, deferral, and release-blocking categories.

## Scope

- Re-run unsupported classification across the expanded corpus.
- Separate release blockers from documented unsupported boundaries.
- Confirm that high-frequency typical-document gaps have owner milestones or
  accepted deferrals.
- Produce the final 1.2 readiness checklist.

## Non-Goals

- Implement every remaining unsupported feature.
- Hide unsupported outcomes from consumer APIs.
- Defer release blockers without a documented decision.

## Deliverables

- Unsupported feature burn-down report.
- Release-blocker and deferral list.
- Updated 1.2 readiness checklist.

## Acceptance Criteria

- Every frequent unsupported feature has a decision.
- Release blockers are measurable and reproducible.
- Accepted deferrals are documented in public support boundaries.

## Validation

- Run native-only `cargo test`.
- Run full unsupported classification.
- Run supported corpus gate.
- Review support matrix and public docs.

## Completion Notes

- Reran the expanded generated corpus unsupported classification:
  227 total, 211 native rendered, 12 typed unsupported, 3 malformed policy
  errors, and 1 encrypted policy error.
- Added fixture-level benchmark evidence for all typed unsupported rows.
- Added strict supported-family gate for `browser-print`, `email-web-archive`,
  and `form`: 43/43 native rendered, 0 fallbacks, 0 errors.
- Updated support matrix with 0199 release-blocker and deferral decisions.
- Report:
  `docs/reports/unsupported-feature-burn-down-2026-06-29.md`.
- Validation:
  - `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/unsupported-0199-classification.json`
  - `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/unsupported-0199-benchmark-fixtures.json`
  - `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family email-web-archive --include-family form --fail-on-fallback --max-edge 160 --output target/unsupported-0199-supported-families.json`
  - `cargo fmt --check`
  - `git diff --check -- docs/reports/native-renderer-support-matrix-2026-06-24.md docs/milestones/0199-unsupported-feature-burn-down-release-candidate-gate.md docs/milestones/README.md docs/reports/unsupported-feature-burn-down-2026-06-29.md`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
