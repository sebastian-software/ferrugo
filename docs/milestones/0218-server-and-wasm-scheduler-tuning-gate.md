# 0218: Server And WASM Scheduler Tuning Gate

Status: todo
Phase: 41
Size: medium
Depends on: 0217

## Goal

Tune the Rust-native rendering scheduler for server batch workloads and WASM
viewer workloads without sacrificing deterministic output or memory budgets.

## Scope

- Profile page scheduling, cancellation, cache reuse, worker limits, and
  backpressure across server and browser profiles.
- Tune batch thumbnail, first-page, navigation, and repeated-render workflows.
- Validate scheduler behavior under constrained memory and parallel workloads.
- Document profile-specific defaults and override boundaries.

## Non-Goals

- Introduce nondeterministic rendering to improve throughput.
- Require threads where a WASM target cannot use them.
- Optimize one profile by regressing another supported profile.

## Deliverables

- Server and WASM scheduler tuning report.
- Profile-specific scheduler budget updates.
- Regression tests for cancellation and deterministic output.

## Acceptance Criteria

- Server and WASM workflows meet throughput and responsiveness budgets.
- Cancellation and backpressure do not leak memory or leave invalid cache state.
- Scheduler defaults are documented per profile.

## Validation

- Run native-only `cargo test`.
- Run server batch rendering benchmark.
- Run WASM viewer performance gate.
- Run cancellation and page-cache reuse tests.
- Run low-memory scheduler profile.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
