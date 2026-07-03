# Stroke Outline Y-Break Fast Path

Date: 2026-07-03

## Summary

This slice reduces the remaining `vector-stress.pdf` cost after stroke distance
route retirement without reintroducing any production distance, span,
axis-span, or row-bucket stroke route.

Generated simple stroke outlines now mark their fill path as safe to skip
scanline-cell edge-intersection y-break scans. The fast path is limited to
single-line outlines and round-polyline outlines whose source centerline has no
non-adjacent intersections. Generic PDF fills and complex/self-intersecting
paths keep the existing y-break handling.

## Performance

Same host, release binary, `--max-edge 160`, 500 iterations:

| Build | Artifact | Mean |
| --- | --- | ---: |
| current `main` (`e4a4fc5`) | `target/issue31-main-vector-stress-refresh-1.json` | 2.134 ms |
| current `main` (`e4a4fc5`) | `target/issue31-main-vector-stress-refresh-2.json` | 2.137 ms |
| guarded y-break fast path | `target/issue31-final-guarded-vector-stress-1.json` | 1.488 ms |
| guarded y-break fast path | `target/issue31-final-guarded-vector-stress-2.json` | 1.494 ms |

The guarded fast path is about 30% faster than the current outline-only route
on this fixture. It is still slower than the earlier retired distance-route
baseline (`0.472 ms` from `docs/reports/stroke-vector-stress-outline-2026-07-03.md`),
so this narrows but does not erase the historical #26 performance caveat.

## Route Audit

Release `trace-native` artifacts:

| Fixture | outline fill | outline joined | axis span | span | row bucket |
| --- | ---: | ---: | ---: | ---: | ---: |
| `dashed-stroke.pdf` | 10 | 0 | 0 | 0 | 0 |
| `line-caps.pdf` | 6 | 0 | 0 | 0 | 0 |
| `line-joins.pdf` | 6 | 6 | 0 | 0 | 0 |
| `vector-paths.pdf` | 2 | 2 | 0 | 0 | 0 |
| `vector-stress.pdf` | 66 | 22 | 0 | 0 | 0 |

The `vector-stress.pdf` trace reports `raster_paths=1.484 ms` with
`coverage_span_calls=2` and `coverage_cell_accumulator_pixels=1954`.

## Visual Diff

Poppler visual diffs used the existing broad vector-stress thresholds
(`--max-mae 16 --max-p95 190 --max-changed-ratio 0.30`):

| Fixture | Status | Changed pixels | Changed ratio | MAE | p95 | Max delta |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `dashed-stroke.pdf` | exact | 0 | 0.000000 | 0.000 | 0 | 0 |
| `line-caps.pdf` | accepted drift | 13 | 0.000903 | 0.033 | 0 | 72 |
| `line-joins.pdf` | accepted drift | 21 | 0.001458 | 0.170 | 0 | 255 |
| `vector-paths.pdf` | accepted drift | 2880 | 0.137405 | 0.283 | 1 | 53 |
| `vector-stress.pdf` | accepted drift | 5559 | 0.289531 | 12.362 | 103 | 160 |

These metrics match the previously accepted stroke-outline drift envelope.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p ferrugo-render round_polyline_stroke_outline_should_only_skip_y_breaks_for_simple_centerlines -- --nocapture
cargo test -p ferrugo-render path_rasterizer_should_draw_generated_vector_stress_fixture -- --nocapture
cargo build --release -p ferrugo --no-default-features
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/issue31-final-guarded-vector-stress-1.json
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/issue31-final-guarded-vector-stress-2.json
target/release/ferrugo visual-diff-poppler fixtures/generated/vector-stress.pdf --max-edge 160 --max-mae 16 --max-p95 190 --max-changed-ratio 0.30 --output target/issue31-guarded-vector-stress-poppler.json
cargo test -p ferrugo-render
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
rg -a -n "point_in_stroke|stroke_row_buckets|rasterize_span_covered_stroke_ranges|distance_to_line|distance_to_segment|row_bucketed_stroke" target/release/ferrugo
```

The release symbol scan returned no matches.
