# Vector-Stress Stroke Outline Routing

Date: 2026-07-03
Issue: #26

## Summary

This slice removes the remaining measured production distance routes from
`vector-stress.pdf` by routing the fixture's joined strokes through fill
outlines:

- thin closed butt-capped rectangle strokes use a compact closed-rectangle
  outline instead of axis-span joins;
- open round-cap/round-join polylines use a single polygonal stroke outline
  instead of the row-bucket predicate route.

The route-retirement goal is met for this fixture, but the local benchmark
shows a real cost. This report documents that cost instead of treating it as a
performance win.

## Route Evidence

Commands:

```bash
cargo run --release -p ferrugo --no-default-features -- trace-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/issue26-vector-audit/vector-stress-before.json
cargo run --release -p ferrugo --no-default-features -- trace-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/issue26-vector-audit/vector-stress-after.json
```

Artifacts:

- `target/issue26-vector-audit/vector-stress-before.json`
- `target/issue26-vector-audit/vector-stress-after.json`

| Counter | Before | After |
| --- | ---: | ---: |
| `outline_fill_calls` | 44 | 66 |
| `outline_axis_line_calls` | 44 | 44 |
| `outline_joined_calls` | 0 | 22 |
| `axis_span_calls` | 20 | 0 |
| `axis_span_join_calls` | 20 | 0 |
| `axis_span_coverage_spans` | 4,016 | 0 |
| `axis_span_raster_spans` | 4,056 | 0 |
| `row_bucket_range_calls` | 2 | 0 |
| `row_bucket_active_range_calls` | 2 | 0 |
| `row_bucket_pixels` | 3,438 | 0 |
| `row_bucket_sample_points` | 13,752 | 0 |
| `row_bucket_line_candidates` | 19,662 | 0 |
| `row_bucket_active_line_refs` | 5,568 | 0 |

After trace dimensions stayed `160x120`, `76,800` output bytes.

The replacement outlines use the fill coverage route:

| Fill counter | After |
| --- | ---: |
| `sampled_calls` | 0 |
| `coverage_span_calls` | 22 |
| `coverage_span_rows` | 1,096 |
| `coverage_span_intervals` | 2,340 |
| `coverage_analytic_edge_pixels` | 6,032 |
| `coverage_cell_accumulator_pixels` | 8,538 |

## Visual Drift

Command:

```bash
cargo run --release -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated/vector-stress.pdf --max-edge 160 --max-mae 60.0 --max-p95 190 --max-changed-ratio 0.55 --timeout 30 --output target/issue26-vector-audit/vector-stress-poppler.json
```

Artifact:

- `target/issue26-vector-audit/vector-stress-poppler.json`

Result: accepted drift against the existing broad vector-stress thresholds.

| Metric | Value |
| --- | ---: |
| Changed pixels | 5,559 |
| Changed ratio | 0.289531 |
| Mean absolute error | 12.362 |
| P95 channel delta | 103 |
| Max channel delta | 160 |
| Native nonwhite pixels | 11,264 |
| Reference nonwhite pixels | 11,352 |

## Performance

Commands:

```bash
# baseline worktree at origin/main
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-vector-audit/vector-stress-benchmark-baseline-200.json

# patched worktree
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-vector-audit/vector-stress-benchmark-after-200.json
```

Artifacts:

- `/private/tmp/ferrugo-issue26-vector-baseline/target/issue26-vector-audit/vector-stress-benchmark-baseline-200.json`
- `target/issue26-vector-audit/vector-stress-benchmark-after-200.json`

| Run | Mean time | Output bytes |
| --- | ---: | ---: |
| Baseline | 0.472 ms | 76,800 |
| Outline route | 2.568 ms | 76,800 |

The outline route is slower on this fixture by `2.096 ms` in this run. The
change is accepted here as route-retirement evidence, not as a performance win.

## Remaining #26 Work

The traced acceptance fixtures now have zero measured production span, axis, or
row-bucket stroke route calls after the line-cap, joined-stroke, and
vector-stress slices:

| Fixture | Outline fills | Joined outlines | Axis-span calls | Span calls | Row-bucket calls |
| --- | ---: | ---: | ---: | ---: | ---: |
| `dashed-stroke.pdf` | 10 | 0 | 0 | 0 | 0 |
| `line-caps.pdf` | 6 | 0 | 0 | 0 | 0 |
| `line-joins.pdf` | 6 | 6 | 0 | 0 | 0 |
| `vector-paths.pdf` | 2 | 2 | 0 | 0 | 0 |
| `vector-stress.pdf` | 66 | 22 | 0 | 0 | 0 |

Remaining cleanup is to make any distance-predicate helpers test-only or remove
them where the fallback tests no longer need them, then capture release-symbol
evidence.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-render stroke_raster_route_summary_should_count_joined_outline_fill_calls -- --nocapture
cargo test -p ferrugo-render stroke_raster_route_summary_should_count_axis_span_join_calls -- --nocapture
cargo test -p ferrugo-render path_rasterizer_should_draw_generated_vector_stress_fixture -- --nocapture
cargo test -p ferrugo-native native_backend_should_render_generated_vector_stress_fixture -- --nocapture
cargo test -p ferrugo-native native_backend_should_synthesize_markup_annotations_without_appearance -- --nocapture
cargo test -p ferrugo-render
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
