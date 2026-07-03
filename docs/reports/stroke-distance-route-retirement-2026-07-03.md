# Stroke Distance Route Retirement

Date: 2026-07-03
Issue: #26

## Summary

This cleanup removes the span, axis-span, row-bucket, and generic
distance-predicate stroke routes from production rendering. The old helpers
remain only under `#[cfg(test)]` where existing oracle tests still compare
their predicates and span builders.

Production stroke rendering now routes visible stroke geometry through fill
outlines:

- joinless strokes use per-line fill outlines, including snapped hairlines;
- joined solid strokes use joined fill outlines, including the component-union
  fallback for cap/join combinations outside the specialized rectangle and
  round-polyline paths.

## Route Evidence

Commands:

```bash
target/release/ferrugo trace-native fixtures/generated/line-caps.pdf --max-edge 160 --output target/issue26-cleanup-audit/line-caps.json
target/release/ferrugo trace-native fixtures/generated/line-joins.pdf --max-edge 160 --output target/issue26-cleanup-audit/line-joins.json
target/release/ferrugo trace-native fixtures/generated/dashed-stroke.pdf --max-edge 160 --output target/issue26-cleanup-audit/dashed-stroke.json
target/release/ferrugo trace-native fixtures/generated/vector-paths.pdf --max-edge 160 --output target/issue26-cleanup-audit/vector-paths.json
target/release/ferrugo trace-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/issue26-cleanup-audit/vector-stress.json
```

| Fixture | Outline fills | Joined outlines | Axis-span calls | Axis-span joins | Span calls | Row-bucket calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `dashed-stroke.pdf` | 10 | 0 | 0 | 0 | 0 | 0 |
| `line-caps.pdf` | 6 | 0 | 0 | 0 | 0 | 0 |
| `line-joins.pdf` | 6 | 6 | 0 | 0 | 0 | 0 |
| `vector-paths.pdf` | 2 | 2 | 0 | 0 | 0 | 0 |
| `vector-stress.pdf` | 66 | 22 | 0 | 0 | 0 | 0 |

## Release Symbol Evidence

Command:

```bash
nm target/release/ferrugo | rg "distance_to_(bounded_)?line|point_in_(stroke|row_bucketed_stroke|single_stroke_line|bounded_stroke_line|join_buckets|join_bucket_candidates|prepared_join_side)|rasterize_(axis_stroke_spans|row_bucketed_stroke_ranges|span_covered_stroke_ranges|simple_line_stroke_spans)|stroke_row_buckets|stroke_join_buckets|axis_stroke_(raster_)?spans|simple_line_stroke_raster_spans|row_bucket_range_raster_candidate"
```

Result: no matches. The release binary was built with:

```bash
cargo build --release -p ferrugo --no-default-features
```

## Visual Drift

Commands:

```bash
target/release/ferrugo visual-diff-poppler fixtures/generated/line-caps.pdf --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue26-cleanup-audit/line-caps-poppler.json
target/release/ferrugo visual-diff-poppler fixtures/generated/line-joins.pdf --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue26-cleanup-audit/line-joins-poppler.json
target/release/ferrugo visual-diff-poppler fixtures/generated/dashed-stroke.pdf --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue26-cleanup-audit/dashed-stroke-poppler.json
target/release/ferrugo visual-diff-poppler fixtures/generated/vector-paths.pdf --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue26-cleanup-audit/vector-paths-poppler.json
target/release/ferrugo visual-diff-poppler fixtures/generated/vector-stress.pdf --max-edge 160 --max-mae 60.0 --max-p95 190 --max-changed-ratio 0.55 --timeout 30 --output target/issue26-cleanup-audit/vector-stress-poppler.json
```

| Fixture | Blockers | Changed pixels | Changed ratio | MAE | P95 delta | Max delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `dashed-stroke.pdf` | 0 | 0 | 0 | 0 | 0 | 0 |
| `line-caps.pdf` | 0 | 13 | 0.000903 | 0.033 | 0 | 72 |
| `line-joins.pdf` | 0 | 21 | 0.001458 | 0.170 | 0 | 255 |
| `vector-paths.pdf` | 0 | 2,880 | 0.137405 | 0.283 | 1 | 53 |
| `vector-stress.pdf` | 0 | 5,559 | 0.289531 | 12.362 | 103 | 160 |

## Performance

Command:

```bash
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue26-cleanup-audit/vector-stress-benchmark-200.json
```

Result: `2.530 ms` mean over 200 iterations, `76,800` output bytes.

This stays in the same slower outline-route band documented by the
vector-stress slice (`0.472 ms` baseline vs `2.568 ms` outline route). The
retirement is therefore supported as removal of production distance routes, not
as a speedup.

## Validation

```bash
cargo fmt --check
cargo build --release -p ferrugo --no-default-features
cargo test -p ferrugo-render
cargo test -p ferrugo-native native_backend_should_render_generated_vector_stress_fixture -- --nocapture
cargo test -p ferrugo-native native_backend_should_synthesize_markup_annotations_without_appearance -- --nocapture
```

Additional workspace validation is listed in the PR.
