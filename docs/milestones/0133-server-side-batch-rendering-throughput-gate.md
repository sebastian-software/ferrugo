# 0133: Server-Side Batch Rendering Throughput Gate

Status: done
Phase: 24
Size: medium
Depends on: 0132

## Goal

Measure and improve native renderer behavior for server-side batch thumbnail
generation across many independent PDF inputs.

## Scope

- Add batch benchmark scenarios for many small and mixed-size documents.
- Track throughput, latency distribution, memory high-water marks, and errors.
- Reuse caches only where isolation and memory limits remain clear.
- Document recommended worker counts and scheduling defaults.

## Non-Goals

- Build a hosted service.
- Add queue infrastructure.
- Share untrusted document state across tenants.

## Deliverables

- Batch throughput benchmark.
- Server-side rendering profile notes.
- Isolation and cache reuse recommendations.

## Acceptance Criteria

- Batch rendering throughput is measured with reproducible fixtures.
- Memory stays bounded across repeated documents.
- Failure reporting identifies input, page, and typed renderer reason.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run batch rendering benchmark.
- Run memory high-water measurement.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `benchmark-batch-native` with worker and in-flight pixel budgets,
  repetitions, latency distribution, throughput, typed per-input outcomes, and
  optional RSS sampling.
- Added `fixtures/server-batch-manifest.tsv` for reproducible server batch
  scenarios across small, mixed-size, image-heavy, repeated-resource, and
  vector-stress fixtures.
- Batch gate rendered 16/16 jobs natively with 0 fallbacks, 0 errors, and 0
  budget failures using two workers.
- Measured throughput: 26.408 jobs/sec; latency p50 26.687 ms, p95 184.736 ms,
  max 184.736 ms.
- RSS sampling in an unsandboxed run reported start 2848 KiB and high-water
  5664 KiB.
- Report: `docs/reports/server-batch-throughput-2026-06-25.md`.
