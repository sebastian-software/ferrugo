# 0095: Large Document Memory Eviction And Spooling

Status: completed
Phase: 16
Size: medium
Depends on: 0094

## Goal

Keep native rendering predictable for large documents by evicting decoded data
and spooling expensive intermediates under explicit policy.

## Scope

- Measure peak memory on multi-page image-heavy and report-heavy documents.
- Add cache eviction for decoded streams, images, glyphs, and form results.
- Evaluate temporary spooling for large intermediates where it reduces peak
  memory without hiding errors.
- Expose configuration for memory ceilings.

## Non-Goals

- Add network-backed storage.
- Optimize for unlimited throughput at the expense of memory.
- Spill sensitive document data without documented opt-in policy.

## Deliverables

- Memory eviction implementation.
- Large-document memory report.
- Configuration documentation.

## Acceptance Criteria

- Large fixture rendering stays within documented memory budgets.
- Eviction decisions are deterministic and observable in diagnostics.
- Temporary storage behavior is explicit and disabled or bounded by default.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run large-document memory measurements.
- Run renderer benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added a page-level decoded image resource budget in addition to the existing
  per-image byte limit.
- Exposed the new image budget plus disabled-by-default spooling policy through
  native memory diagnostics and CLI JSON.
- Kept temporary spooling disabled with a `0` byte budget pending a future
  explicit privacy and cleanup policy.
- Updated `docs/policies/renderer-memory-budgets.md`.
- Evidence report: `docs/reports/large-document-memory-2026-06-25.md`.
