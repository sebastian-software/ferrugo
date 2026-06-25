# 0218: Server And WASM Scheduler Tuning Gate

Status: todo
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

Empty until done.
