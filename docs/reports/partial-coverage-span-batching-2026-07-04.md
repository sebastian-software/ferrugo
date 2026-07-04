# Partial-Coverage Span Batching

Status: accepted.
Date: 2026-07-04.

Issue: #113.

## Decision

Analytic and scanline-cell edge coverage now flows through row coverage-alpha
span batches instead of writing every nonzero edge pixel through the per-pixel
coverage writer. The implementation keeps the scalar source-over math as the
parity path and adds a `ferrugo-simd` scalar coverage-alpha row kernel as the
future vectorization boundary for #110.

## Trace Evidence

Command:

```sh
cargo run -p ferrugo --release --no-default-features -- trace-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/issue113-after-trace.json
```

`vector-stress.pdf` reported the following fill-route counters after the
change:

| Counter | Value |
| --- | ---: |
| `coverage_span_calls` | 2 |
| `coverage_span_rows` | 52 |
| `coverage_full_span_runs` | 12 |
| `coverage_full_pixels` | 18 |
| `coverage_partial_span_runs` | 340 |
| `coverage_partial_span_pixels` | 1078 |
| `coverage_analytic_edge_pixels` | 1376 |
| `coverage_cell_accumulator_pixels` | 1954 |
| `coverage_sampled_edge_pixels` | 0 |

The new `coverage_partial_span_*` counters are the routing proof: partial edge
pixels are now visible as span-batched work instead of only analytic edge pixel
visits.

## Benchmark Evidence

Before run from `main` at `bdfa211`; after run from
`codex/issue-113-partial-coverage-spans`.

Command shape:

```sh
cargo run -p ferrugo --release --no-default-features -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --include-family report/vector \
  --backend native \
  --mode hot-render \
  --max-edge 160 \
  --iterations 50 \
  --warmup 5 \
  --max-cov 0.50 \
  --timeout 30
```

| Fixture | Before p95 | After p95 | p95 ratio | Before mean | After mean | Before CoV | After CoV |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `prepress-trim-bleed-marks.pdf` | 1.572 ms | 1.544 ms | 0.982x | 1.501 ms | 1.462 ms | 0.032 | 0.031 |
| `technical-hatch-clipping.pdf` | 3.569 ms | 3.413 ms | 0.956x | 3.358 ms | 3.328 ms | 0.039 | 0.017 |
| `technical-linework-dimensions.pdf` | 0.736 ms | 0.655 ms | 0.890x | 0.648 ms | 0.621 ms | 0.067 | 0.040 |
| `vector-stress.pdf` | 1.934 ms | 1.847 ms | 0.955x | 1.761 ms | 1.720 ms | 0.052 | 0.041 |

This is a scoped renderer hot-path improvement, not a public cross-renderer
performance claim. The benchmark uses a local release build and generated
`report/vector` fixtures to validate the optimization direction before #110
adds architecture-specific coverage span kernels.

## Validation

```sh
cargo test -p ferrugo-simd
cargo test -p ferrugo-render
cargo check -p ferrugo --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```
