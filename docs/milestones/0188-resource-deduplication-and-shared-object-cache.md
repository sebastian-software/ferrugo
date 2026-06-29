# 0188: Resource Deduplication And Shared Object Cache

Status: done
Phase: 35
Size: medium
Depends on: 0187

## Goal

Reduce repeated work and memory churn by caching shared fonts, images, color
transforms, and decoded resources across pages.

## Scope

- Audit duplicated resource decoding across multi-page documents.
- Add bounded caches for shared immutable renderer resources.
- Define cache keys that avoid retaining unbounded page state.
- Measure hit rates, memory cost, and eviction behavior.

## Non-Goals

- Add a global process-wide cache with unbounded lifetime.
- Cache security-sensitive decrypted content beyond document scope.
- Optimize rare resources before common page-level reuse.

## Deliverables

- Shared object cache design or implementation.
- Cache budget and eviction documentation.
- Multi-page benchmark report.

## Acceptance Criteria

- Repeated fonts and images are decoded once where safe.
- Cache memory is bounded and observable.
- Multi-page rendering improves or remains neutral without fidelity regression.

## Validation

- Run native-only `cargo test`.
- Run multi-page benchmark profiles.
- Run cache eviction tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Changed the bounded multi-page renderer to load the document and page tree
  once per render request, then share that immutable object graph across scoped
  page workers.
- Preserved the linearized first-page fast path for single-page page-zero
  requests while forcing multi-page requests through the full classic loader so
  later pages never read an incomplete first-page-only object table.
- Kept decoded page resources pass-local; no process-wide or tenant-shared
  cache was introduced.
- Added `fixtures/shared-resource-cache-manifest.tsv` and
  `docs/reports/shared-resource-cache-2026-06-29.md` for repeated font/image,
  long-document, repeated XObject, Type3, and ICC cache evidence.
- Follow-up: a true document-session decoded font/image cache still needs
  explicit byte accounting and eviction before it can replace page-local decode
  caches.
