# Stroke Line-Cap Outline Routing

Date: 2026-07-03
Issue: #26

## Summary

This slice routes the remaining non-hairline, joinless `line-caps.pdf` strokes
through fill outlines instead of the simple-line span predicate path. It is a
narrow outliner-retirement step for round-capped axis-aligned lines; it does not
close #26.

The runtime route counters now show zero production span-covered calls for the
`line-caps` fixture.

## Route Evidence

Commands:

```bash
cargo run --release -p ferrugo --no-default-features -- trace-native fixtures/generated/line-caps.pdf --max-edge 160 --output target/issue26-route-audit/line-caps.json
cargo run --release -p ferrugo --no-default-features -- trace-native fixtures/generated/line-caps.pdf --max-edge 160 --output target/issue26-route-audit/line-caps-after.json
```

Artifacts:

- `target/issue26-route-audit/line-caps.json`
- `target/issue26-route-audit/line-caps-after.json`

| Counter | Before | After |
| --- | ---: | ---: |
| `outline_fill_calls` | 4 | 6 |
| `outline_axis_line_calls` | 4 | 6 |
| `span_covered_calls` | 2 | 0 |
| `span_from_start_calls` | 2 | 0 |
| `span_coverage_spans` | 16 | 0 |
| `span_pixels` | 672 | 0 |
| `simple_line_span_routed_items` | 6 | 0 |

After trace dimensions stayed `120x120`, `57,600` output bytes.

## Visual Drift

Command:

```bash
cargo run --release -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated/line-caps.pdf --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue26-route-audit/line-caps-poppler.json
```

Artifact:

- `target/issue26-route-audit/line-caps-poppler.json`

Result: accepted drift against Poppler.

| Metric | Value |
| --- | ---: |
| Changed pixels | 13 |
| Changed ratio | 0.000903 |
| Mean absolute error | 0.033 |
| P95 channel delta | 0 |
| Max channel delta | 72 |
| Native nonwhite pixels | 992 |
| Reference nonwhite pixels | 993 |

## Performance

Commands:

```bash
# baseline worktree at origin/main
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/line-caps.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-route-audit/line-caps-benchmark-baseline-200.json

# patched worktree
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/line-caps.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-route-audit/line-caps-benchmark-after-200.json
```

Artifacts:

- `/private/tmp/ferrugo-issue26-baseline/target/issue26-route-audit/line-caps-benchmark-baseline-200.json`
- `target/issue26-route-audit/line-caps-benchmark-after-200.json`

| Run | Mean time | Output bytes |
| --- | ---: | ---: |
| Baseline | 0.037 ms | 57,600 |
| Outline route | 0.065 ms | 57,600 |

The outline route is slower on this small fixture by `0.028 ms` in this run.
That is documented as the cost of retiring this production distance predicate
slice, not as a performance win.

## Remaining #26 Work

This slice removes the simple-line span route from `line-caps.pdf` only. The
initial route audit still showed remaining non-outline stroke work elsewhere:

| Fixture | Remaining routes |
| --- | --- |
| `line-joins.pdf` | `axis_span_calls=6`, `axis_span_join_calls=6` |
| `vector-stress.pdf` | `axis_span_calls=20`, `axis_span_join_calls=20`, `row_bucket_range_calls=2` |
| `vector-paths.pdf` | joined outlines are already used for the measured strokes |
| `dashed-stroke.pdf` | outline fills are already used for the measured strokes |

The release-symbol cleanup and full production removal of distance predicate
routes remain open.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-render stroke_shape_summary_should -- --nocapture
cargo test -p ferrugo-render stroke_raster_route_summary_should_count -- --nocapture
cargo test -p ferrugo-render rasterize_paths_should_apply_stroke_line_caps -- --nocapture
cargo test -p ferrugo-native native_backend_should_render_generated_line_caps_fixture -- --nocapture
cargo test -p ferrugo-render
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
