# 0219: Unsupported Feature SLA And Consumer Migration Guide

Status: done
Phase: 41
Size: small
Depends on: 0218

## Goal

Define the consumer-facing service level for unsupported PDF features and
document migration guidance for applications that need predictable native-only
renderer behavior.

## Scope

- Consolidate unsupported categories, diagnostics, severity, retry behavior, and
  fallback recommendations.
- Define which unsupported features are release blockers, documented limits, or
  backlog candidates.
- Write migration guidance for applications previously depending on PDFium.
- Add examples for handling typed unsupported outcomes without inspecting
  internal renderer state.

## Non-Goals

- Promise support for every PDF feature.
- Encourage applications to ship private PDFium fallback paths.
- Expose unstable internal diagnostics as public API.

## Deliverables

- Unsupported feature SLA.
- Consumer migration guide.
- Public diagnostic example updates.

## Acceptance Criteria

- Consumers can distinguish unsupported, degraded, failed, and successful
  outcomes through stable APIs.
- Migration guidance covers common native-only deployment profiles.
- Release-blocking unsupported categories are explicit.

## Validation

- Run documentation link checks.
- Run public API examples.
- Run native-only `cargo test`.
- Run unsupported diagnostic snapshot tests.
- Run package dry-runs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Added `docs/policies/unsupported-feature-sla.md` to define stable
  unsupported, degraded, failed, and successful outcome handling.
- Added `docs/guides/native-only-consumer-migration.md` with native-only build,
  deployment-profile, and public API routing guidance.
- Updated `docs/errors.md` to align the bucket table with
  `pdfrust_thumbnail::STABLE_UNSUPPORTED_FEATURE_BUCKETS`.
- Added `scripts/check_unsupported_feature_sla.sh` to validate stable bucket
  coverage and local policy links.
- Added a public facade test showing consumer routing by class first and bucket
  second.
- Report:
  `docs/reports/unsupported-feature-sla-consumer-migration-2026-06-29.md`.

Validation:

- `bash scripts/check_unsupported_feature_sla.sh`
- `cargo test -p pdfrust-thumbnail consumer_migration -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/unsupported-0219-classification.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family email-web-archive --include-family form --fail-on-fallback --max-edge 160 --output target/unsupported-0219-supported-families.json`
- `cargo package -p pdfrust-thumbnail --allow-dirty --no-verify --list`
- `cargo package -p pdfrust-cli --allow-dirty --no-verify --list`
- `cargo fmt --check`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `git diff --check -- crates/pdfrust-thumbnail/src/lib.rs docs/errors.md docs/packaging.md docs/policies/unsupported-feature-sla.md docs/guides/native-only-consumer-migration.md docs/reports/unsupported-feature-sla-consumer-migration-2026-06-29.md scripts/check_unsupported_feature_sla.sh docs/milestones/README.md docs/milestones/0219-unsupported-feature-sla-and-consumer-migration-guide.md`
