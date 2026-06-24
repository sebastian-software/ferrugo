# 0078: Renderer Benchmark Suite And Budgets

Status: todo
Phase: 13
Size: medium
Depends on: 0077

## Goal

Turn renderer performance and memory expectations into repeatable benchmarks
that guide PDFium replacement decisions.

## Scope

- Add benchmarks for text-heavy, image-heavy, vector-heavy, and mixed documents.
- Capture render time, peak allocation proxies, output dimensions, and fallback
  counts.
- Define budgets for CI-friendly smoke benchmarks and local deep benchmarks.
- Compare native results against PDFium where available.

## Non-Goals

- Make noisy microbenchmarks release-blocking.
- Require PDFium for every benchmark.
- Optimize without a measured regression or bottleneck.

## Deliverables

- Benchmark harness and fixture categories.
- Performance budget documentation.
- Baseline numbers for native and PDFium backends.

## Acceptance Criteria

- Developers can reproduce renderer benchmark results locally.
- Performance changes are tied to document categories.
- Budget failures identify whether time, memory, or fallback rate regressed.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run smoke benchmarks and at least one deep local benchmark.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
