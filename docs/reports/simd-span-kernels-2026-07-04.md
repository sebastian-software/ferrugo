# SIMD span kernels - 2026-07-04

Issue: #110

## Summary

This change adds runtime-dispatched integer row kernels for opaque source-over coverage-alpha spans in `ferrugo-simd`, with scalar fallback and trace-visible backend counters in native render traces.

The hot path changed in two steps:

- SIMD-sized opaque coverage-alpha spans dispatch to NEON on aarch64 and AVX2/SSE2 on x86/x86_64 after runtime CPU feature checks.
- Short coverage-alpha spans stay on the historical floating-point scalar oracle, preserving existing renderer byte parity for small edge spans.

## Runtime Dispatch Proof

Command:

```sh
cargo run -p ferrugo --release --no-default-features -- \
  trace-native fixtures/generated/academic-references-appendix.pdf \
  --max-edge 240 \
  --output target/issue110-simd-trace-final.json
```

Trace counters:

| Counter | Value |
| --- | ---: |
| `coverage_partial_span_runs` | 14 |
| `coverage_partial_span_pixels` | 1,950 |
| `coverage_simd_row_kernel_pixels` | 1,950 |
| `coverage_scalar_row_kernel_pixels` | 0 |
| `coverage_analytic_edge_pixels` | 1,950 |
| `coverage_cell_accumulator_pixels` | 1,950 |

The trace was captured on aarch64, so SIMD dispatch selected the NEON kernel.

## Benchmark Matrix

Commands:

```sh
cargo run -p ferrugo --release --no-default-features -- \
  benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --include-family report/vector \
  --backend native \
  --mode hot-render \
  --max-edge 160 \
  --iterations 50 \
  --warmup 5 \
  --max-cov 0.50 \
  --timeout 30 \
  --output target/issue110-after-vector-matrix-final.json \
  --report target/issue110-after-vector-matrix-final.md \
  --artifact-dir /private/tmp/ferrugo-issue110-after-vector-final-artifacts

cargo run -p ferrugo --release --no-default-features -- \
  benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --include-family small-text \
  --include-family office-export \
  --backend native \
  --mode hot-render \
  --max-edge 160 \
  --iterations 50 \
  --warmup 5 \
  --max-cov 0.50 \
  --timeout 30 \
  --output target/issue110-after-text-matrix-final.json \
  --report target/issue110-after-text-matrix-final.md \
  --artifact-dir /private/tmp/ferrugo-issue110-after-text-final-artifacts
```

Before/after p95 hot-render results:

| Fixture | Family | Before p95 ms | After p95 ms | Delta |
| --- | --- | ---: | ---: | ---: |
| `prepress-trim-bleed-marks.pdf` | `report/vector` | 1.931 | 1.554 | 19.5% faster |
| `technical-hatch-clipping.pdf` | `report/vector` | 3.392 | 3.414 | 0.6% slower |
| `technical-linework-dimensions.pdf` | `report/vector` | 0.637 | 0.656 | 3.0% slower |
| `vector-stress.pdf` | `report/vector` | 1.829 | 1.862 | 1.8% slower |
| `office-report-header-footer-link.pdf` | `office-export` | 0.657 | 0.615 | 6.4% faster |
| `text-page.pdf` | `small-text` | 0.047 | 0.042 | 10.6% faster |

All records rendered without fallback, missing tools, errors, or CoV threshold failures. RSS sampling and hot reference-engine comparisons were unavailable for this local run.

## Validation

Executed:

```sh
cargo test -p ferrugo-simd
cargo test -p ferrugo-render
cargo check -p ferrugo --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```
