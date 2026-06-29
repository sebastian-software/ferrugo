# 0197: Serverless Cold Start And Binary Size Budget

Status: done
Phase: 37
Size: medium
Depends on: 0195

## Goal

Keep serverless and short-lived batch rendering practical by bounding binary
size, startup time, and first-render latency without PDFium runtime baggage.
This remains on the primary server-side rendering path and does not depend on
WASM/mobile profile readiness.

## Scope

- Measure cold start and first-render latency for native-only builds.
- Audit feature flags, dependencies, and embedded assets for size impact.
- Add package profiles for serverless thumbnail use.
- Document tradeoffs between size, codec coverage, and performance.

## Non-Goals

- Optimize every deployment platform.
- Remove features required for typical documents.
- Reintroduce dynamic PDFium distribution.

## Deliverables

- Serverless cold-start report.
- Binary size budget and package profile.
- Dependency and feature-flag follow-up list.

## Acceptance Criteria

- Native-only artifacts meet documented size and startup budgets.
- Optional heavyweight features are controlled by explicit feature flags.
- First-render latency is measured and reproducible.

## Validation

- Run release builds for target package profiles.
- Run cold-start benchmark script.
- Run package dry-runs.
- Run native-only `cargo test`.

## Completion Notes

- Added the explicit Cargo `serverless` profile for native-only short-lived
  workers.
- Added `scripts/measure_serverless_profile.sh` to build the profile, inspect
  the CLI package file list, and measure binary size, startup, and first-render
  latency.
- Documented default budgets in `docs/packaging.md` and the measurement flow in
  `docs/benchmarks.md`.
- Report:
  `docs/reports/serverless-cold-start-and-binary-size-2026-06-29.md`.
- Validation:
  - `bash scripts/measure_serverless_profile.sh`
  - `cargo fmt --check`
  - `git diff --check -- Cargo.toml scripts/measure_serverless_profile.sh docs/packaging.md docs/benchmarks.md docs/milestones/0197-serverless-cold-start-and-binary-size-budget.md docs/milestones/README.md docs/reports/serverless-cold-start-and-binary-size-2026-06-29.md`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
