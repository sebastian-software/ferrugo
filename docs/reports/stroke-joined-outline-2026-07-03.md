# Stroke Joined-Outline Routing

Date: 2026-07-03
Issue: #26

## Summary

This slice routes eligible non-hairline, solid, butt-capped open joined strokes
through joined fill outlines before axis-span rasterization. It removes the
`line-joins.pdf` production axis-span join route and keeps square-cap joined
strokes on the existing axis-span route as coverage for that remaining
fallback class.

This does not close #26. `vector-stress.pdf` still has axis-span join and
row-bucket routes after this change.

## Route Evidence

Commands:

```bash
cargo run --release -p ferrugo --no-default-features -- trace-native fixtures/generated/line-joins.pdf --max-edge 160 --output target/issue26-joined-audit/line-joins-before.json
cargo run --release -p ferrugo --no-default-features -- trace-native fixtures/generated/line-joins.pdf --max-edge 160 --output target/issue26-joined-audit/line-joins-after.json
```

Artifacts:

- `target/issue26-joined-audit/line-joins-before.json`
- `target/issue26-joined-audit/line-joins-after.json`

| Counter | Before | After |
| --- | ---: | ---: |
| `outline_fill_calls` | 0 | 6 |
| `outline_joined_calls` | 0 | 6 |
| `axis_span_calls` | 6 | 0 |
| `axis_span_join_calls` | 6 | 0 |
| `axis_span_coverage_spans` | 408 | 0 |
| `axis_span_raster_spans` | 408 | 0 |
| `axis_span_routed_items` | 6 | 0 |

After trace dimensions stayed `120x120`, `57,600` output bytes.

The joined outline route uses coverage fill after this slice:

| Fill counter | After |
| --- | ---: |
| `sampled_calls` | 0 |
| `coverage_span_calls` | 6 |
| `coverage_span_rows` | 204 |
| `coverage_span_intervals` | 316 |

## Visual Drift

Command:

```bash
cargo run --release -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated/line-joins.pdf --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue26-joined-audit/line-joins-poppler.json
```

Artifact:

- `target/issue26-joined-audit/line-joins-poppler.json`

Result: accepted drift against Poppler.

| Metric | Value |
| --- | ---: |
| Changed pixels | 21 |
| Changed ratio | 0.001458 |
| Mean absolute error | 0.170 |
| P95 channel delta | 0 |
| Max channel delta | 255 |
| Native nonwhite pixels | 1,427 |
| Reference nonwhite pixels | 1,433 |

## Performance

Commands:

```bash
# baseline worktree at origin/main
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/line-joins.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-joined-audit/line-joins-benchmark-baseline-200.json

# patched worktree
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/line-joins.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-joined-audit/line-joins-benchmark-after-200.json
```

Artifacts:

- `/private/tmp/ferrugo-issue26-joins-baseline/target/issue26-joined-audit/line-joins-benchmark-baseline-200.json`
- `target/issue26-joined-audit/line-joins-benchmark-after-200.json`

| Run | Mean time | Output bytes |
| --- | ---: | ---: |
| Baseline | 0.115 ms | 57,600 |
| Joined outline route | 0.182 ms | 57,600 |

The joined outline route is slower on this small fixture by `0.067 ms` in this
run. The earlier sampled-fill variant measured `0.699 ms`; keeping opaque
joined outlines on coverage fill avoids that larger cost while still retiring
the production axis-span join route for this fixture.

## Remaining #26 Work

The current `vector-stress.pdf` trace still reports:

| Counter | Current |
| --- | ---: |
| `axis_span_calls` | 20 |
| `axis_span_join_calls` | 20 |
| `row_bucket_range_calls` | 2 |
| `row_bucket_active_range_calls` | 2 |

The release-symbol cleanup and full production removal of remaining distance
predicate routes remain open.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-render stroke_raster_route_summary_should_count_joined_outline_fill_calls -- --nocapture
cargo test -p ferrugo-render stroke_raster_route_summary_should_count_axis_span_join_calls -- --nocapture
cargo test -p ferrugo-render stroke_shape_summary_should_count_axis_span_join_candidates -- --nocapture
cargo test -p ferrugo-render
cargo test -p ferrugo-native native_backend_should_render_generated_line_joins_fixture -- --nocapture
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
