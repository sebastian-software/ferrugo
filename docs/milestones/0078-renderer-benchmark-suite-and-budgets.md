# 0078: Renderer Benchmark Suite And Budgets

Status: done
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

Completed on 2026-06-24.

- Added the reusable renderer benchmark harness in
  `pdfrust-cli benchmark-native` and `pdfrust-cli benchmark-pdfium`.
- Added JSON benchmark reports with backend, budget config, summary, family
  aggregates, fixture outcomes, and typed budget violations.
- Documented benchmark commands and budget policy in `docs/benchmarks.md`.
- Recorded native, PDFium, and deep-local baselines in
  `docs/reports/renderer-benchmark-suite-2026-06-24.md`.
- Validation passed:
  `cargo fmt --check`,
  `cargo test -p pdfrust-cli benchmark -- --nocapture`,
  `cargo check`,
  `cargo test`,
  `cargo clippy --all-targets --all-features -- -D warnings`.

Implementation commit:

- `12254e5 feat: add renderer benchmark harness`
