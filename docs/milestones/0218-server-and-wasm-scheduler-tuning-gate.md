# 0218: Server And WASM Scheduler Tuning Gate

Status: done
Phase: 41
Size: medium
Depends on: 0216

## Goal

Tune the Rust-native rendering scheduler for server batch workloads first, with
WASM viewer behavior treated as a secondary profile check unless it exposes a
shared scheduling correctness, cancellation, or memory defect.

## Scope

- Profile page scheduling, cancellation, cache reuse, worker limits, and
  backpressure across server and browser profiles.
- Tune batch thumbnail, first-page, navigation, and repeated-render workflows.
- Validate scheduler behavior under constrained memory and parallel workloads.
- Document profile-specific defaults and override boundaries.
- Keep server-side batch throughput and isolation as the primary release gate.

## Non-Goals

- Introduce nondeterministic rendering to improve throughput.
- Require threads where a WASM target cannot use them.
- Optimize one profile by regressing another supported profile.
- Let WASM responsiveness block server-side PDFium replacement unless the same
  issue affects shared scheduler correctness or bounded resources.

## Deliverables

- Server and WASM scheduler tuning report.
- Profile-specific scheduler budget updates.
- Regression tests for cancellation and deterministic output.

## Acceptance Criteria

- Server workflows meet throughput, isolation, and responsiveness budgets.
- WASM scheduler behavior is measured and classified as secondary unless it
  reveals shared scheduler defects.
- Cancellation and backpressure do not leak memory or leave invalid cache state.
- Scheduler defaults are documented per profile.

## Validation

- Run native-only `cargo test`.
- Run server batch rendering benchmark.
- Run WASM viewer performance gate as a secondary profile check.
- Run cancellation and page-cache reuse tests.
- Run low-memory scheduler profile.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Added `fixtures/scheduler-tuning-profile-matrix.tsv` with server throughput,
  high-page-count cancellation, low-memory batch, repeat-cache, and WASM smoke
  scheduler profiles.
- Added `scripts/check_scheduler_tuning_matrix.sh` to validate profile
  coverage, blocking scopes, target-local artifacts, and budget notes.
- Added a CLI regression test proving unschedulable batch pixel budgets fail
  with a typed budget error before work is scheduled.
- Updated server batch and native backend documentation with the 0218 scheduler
  default profiles and override boundaries.
- Report: `docs/reports/server-wasm-scheduler-tuning-2026-06-29.md`.

Validation:

- `bash scripts/check_scheduler_tuning_matrix.sh`
- `cargo test -p pdfrust-cli batch_benchmark -- --nocapture`
- `cargo test -p pdfrust-cli repeat_benchmark -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 3 --pages-per-input 1 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/scheduler-0218-server-batch.json`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 3 --pages-per-input 12 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --cancel-after-jobs 20 --output target/scheduler-0218-cancellation.json`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 2 --pages-per-input 1 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/scheduler-0218-low-memory-batch.json`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --repetitions 2 --max-edge 120 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/scheduler-0218-repeat-cache.json`
- `bash scripts/check_wasm_smoke.sh`
- `cargo fmt --check`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `git diff --check -- crates/pdfrust-cli/src/main.rs fixtures/scheduler-tuning-profile-matrix.tsv scripts/check_scheduler_tuning_matrix.sh docs/policies/server-batch-rendering.md docs/backend/native.md docs/milestones/README.md docs/milestones/0218-server-and-wasm-scheduler-tuning-gate.md docs/reports/server-wasm-scheduler-tuning-2026-06-29.md`
