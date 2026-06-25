# 0133: Server-Side Batch Rendering Throughput Gate

Status: todo
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

Empty until done.
