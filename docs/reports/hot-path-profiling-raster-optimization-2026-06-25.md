# Hot Path Profiling Raster Optimization

Date: 2026-06-25
Milestone: 0096

## Scope

This pass optimized the native path rasterizer hot path measured on the generated
`vector-stress.pdf` fixture. The change keeps the rasterizer unsafe-free and
does not add intermediate buffers or persistent caches.

## Profiling Evidence

- Fixture: `fixtures/generated/vector-stress.pdf`
- Before benchmark artifact:
  `target/hotpath-0096-vector-stress-before.json`
- After benchmark artifact:
  `target/hotpath-0096-vector-stress-after.json`
- Profiling capture:
  `target/hotpath-0096-vector-stress.sample.txt`

The `sample` capture still shows `fill_path`, `stroke_path`,
`point_in_active_clips`, `point_in_path`, `point_in_stroke`, and
`point_in_join` as the relevant geometry costs after the loop bound reduction.

## Optimization

`fill_path`, `fill_path_with_tiling_pattern`, and `stroke_path` previously
visited every pixel in the raster device for each path item. The optimized path
computes clipped device-pixel bounds from the flattened path or stroke geometry
and only samples pixels that can be affected by the item.

Stroke bounds include radius padding so caps and joins remain covered. Off-page
paths collapse to `None` and return early without weakening clipping, blending,
or raster bounds checks.

## Benchmark Delta

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated/vector-stress.pdf --manifest fixtures/corpus-manifest.tsv --max-edge 320 --iterations 5 --max-ms 10000 --max-output-bytes 4194304 --output target/hotpath-0096-vector-stress-after.json
```

Results:

| Run | Mean ms | Output bytes | Budget failures |
| --- | ---: | ---: | ---: |
| Before | 2833.258 | 76800 | 0 |
| After | 184.189 | 76800 | 0 |

The representative vector-stress render is about 15.4x faster in the local dev
benchmark while producing the same output size and no budget violations.

## Safety Notes

- No unsafe code was introduced.
- The optimization narrows iteration ranges only; per-pixel sample coverage,
  active clipping, fill rules, stroke joins, caps, alpha, and blending remain in
  the existing code path.
- Empty or fully off-device bounds return early and avoid unnecessary work.

## Validation

- `cargo fmt`
- `cargo test -p ferrugo-render path_rasterizer -- --nocapture`
- `cargo test -p ferrugo-render pixel_bounds -- --nocapture`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated/vector-stress.pdf --manifest fixtures/corpus-manifest.tsv --max-edge 320 --iterations 5 --max-ms 10000 --max-output-bytes 4194304 --output target/hotpath-0096-vector-stress-after.json`
- `sample <pid> 2 -file target/hotpath-0096-vector-stress.sample.txt`
