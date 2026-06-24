# 0095: Large Document Memory Eviction And Spooling

Status: todo
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

Empty until done.
