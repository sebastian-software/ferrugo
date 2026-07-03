# Stroke Outline Y-Break Fast Path

Date: 2026-07-03

## Summary

This slice reduces the remaining `vector-stress.pdf` cost after stroke distance
route retirement without reintroducing any production distance, span,
axis-span, or row-bucket stroke route.

Generated simple stroke outlines now mark their fill path as safe to skip
scanline-cell edge-intersection y-break scans. The fast path is limited to
single-line outlines and round-polyline outlines whose source centerline has no
non-adjacent intersections **and whose offset outline has no self-intersections
strictly between vertex rows**. Generic PDF fills and complex/self-intersecting
paths keep the existing y-break handling.

The outline-geometry guard was added after review: the centerline check alone
was unsound. At acute joins the inner offset edges of adjacent segments cross
mid-edge (no vertex at the crossing, so vertex y-breaks miss it), and the slab
decomposition then computes wrong coverage — reproduced as coverage 127/255
instead of 216/255 at the join pixel for polyline `(2,2)→(12,3)→(2,6)` with
radius 1.0. Crossings that sit exactly on a vertex row (e.g. axis-aligned
right-angle joins) remain safe because every vertex y is a slab boundary, so
the guard only rejects crossings strictly between vertex rows. Both pairwise
scans are additionally capped at 256 edges (`STROKE_INTERSECTION_SCAN_MAX_EDGES`)
to avoid a quadratic cliff on long polylines; longer inputs conservatively keep
the y-break scans.

## Performance

Same host, release binary, `--max-edge 160`, 500 iterations:

| Build | Artifact | Mean |
| --- | --- | ---: |
| current `main` (`e4a4fc5`) | `target/issue31-main-vector-stress-refresh-1.json` | 2.134 ms |
| current `main` (`e4a4fc5`) | `target/issue31-main-vector-stress-refresh-2.json` | 2.137 ms |
| centerline-only guard (unsound, superseded) | `target/issue31-final-guarded-vector-stress-1.json` | 1.488 ms |
| centerline-only guard (unsound, superseded) | `target/issue31-final-guarded-vector-stress-2.json` | 1.494 ms |
| outline-geometry guard (this slice) | `target/pr48-fix-vector-stress-1.json` | 2.086 ms |
| outline-geometry guard (this slice) | `target/pr48-fix-vector-stress-2.json` | 2.052 ms |

Honest accounting: most of the earlier 30% win came from strokes whose y-break
scans were skipped unsoundly — the `vector-stress.pdf` polylines have joins
whose outline crossings fall between vertex rows, so the corrected guard keeps
their scans enabled. The sound remainder is a ~2–4% improvement on this
fixture (2.134 → 2.052–2.086 ms). The structural way to recover the full win
safely is to compute the edge-intersection y-breaks once per path (cached in
`FillEdgeCoverage`) instead of rescanning per row — that benefits all fills,
not just generated stroke outlines, and is left as a follow-up. The historical
#26 performance caveat versus the retired distance-route baseline (`0.472 ms`)
therefore remains open.

## Route Audit

Release `trace-native` artifacts:

| Fixture | outline fill | outline joined | axis span | span | row bucket |
| --- | ---: | ---: | ---: | ---: | ---: |
| `dashed-stroke.pdf` | 10 | 0 | 0 | 0 | 0 |
| `line-caps.pdf` | 6 | 0 | 0 | 0 | 0 |
| `line-joins.pdf` | 6 | 6 | 0 | 0 | 0 |
| `vector-paths.pdf` | 2 | 2 | 0 | 0 | 0 |
| `vector-stress.pdf` | 66 | 22 | 0 | 0 | 0 |

The route audit is unchanged by the corrected guard: no retired stroke route
comes back, only the share of outline fills that skip the y-break scan shrinks.

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
cargo test -p ferrugo-render round_polyline_stroke_outline_should_only_skip_y_breaks_for_simple_outlines -- --nocapture
cargo test -p ferrugo-render round_polyline_stroke_outline_should_not_skip_y_breaks_past_the_scan_cap -- --nocapture
cargo test -p ferrugo-render path_rasterizer_should_draw_generated_vector_stress_fixture -- --nocapture
cargo build --release -p ferrugo --no-default-features
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/pr48-fix-vector-stress-1.json
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/pr48-fix-vector-stress-2.json
target/release/ferrugo visual-diff-poppler fixtures/generated/vector-stress.pdf --max-edge 160 --max-mae 16 --max-p95 190 --max-changed-ratio 0.30 --output target/pr48-fix-vector-stress-poppler.json
cargo test -p ferrugo-render
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
rg -a -n "point_in_stroke|stroke_row_buckets|rasterize_span_covered_stroke_ranges|distance_to_line|distance_to_segment|row_bucketed_stroke" target/release/ferrugo
```

The release symbol scan returned no matches. Poppler visual diffs after the
corrected guard are byte-identical to the table above (`dashed-stroke` exact;
13 / 21 / 2880 / 5559 changed pixels on the drift fixtures).
