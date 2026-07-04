# Render Performance Workstream

Date: 2026-07-04
Issues: #49, #50, #51

## Summary

This workstream closes the next practical pass over the already-started render
performance issues without preserving the earlier aspirational targets as hard
gates. The accepted bar for this pass is narrower: land measured improvements
where the current architecture has low-risk headroom, keep route and parity
tests green, and document noisy or exhausted paths honestly.

The changes are:

- cache sorted path vertex y-breaks alongside cached edge-intersection
  y-breaks in `FillEdgeCoverage`, so scanline slab setup no longer scans every
  path vertex on every row;
- extend the `ferrugo-simd` safe row-kernel boundary from opaque normal
  source-over rows to general normal source-over rows, and make the scalar row
  kernels inlineable across the render boundary;
- reduce direct scanned-image row overhead by precomputing row sample indices
  and by using a branch-light inner loop when target columns map to unique
  source samples.

No unsafe code was added. `ferrugo-render` and `ferrugo-native` keep their
current safety posture.

## Measurements

All measurements used release binaries on the same machine. Baseline is
`origin/main` at `c541811`; branch is `codex/issues-49-50-51-perf`.

| Fixture | Area | Baseline | Branch | Result |
| --- | --- | ---: | ---: | ---: |
| `vector-stress.pdf`, max edge 160, 300 iterations | #49 y-break/fill | 1.782 ms | 1.714 ms | 3.8% faster |
| `vector-stress.pdf`, max edge 320, 200 iterations | #49 y-break/fill | 1.739 ms | 1.714 ms | 1.4% faster |
| `blend-modes.pdf`, max edge 320, 500 iterations | #50 row-kernel boundary | 0.131 ms | 0.104 ms | 20.6% faster |
| `transparency-alpha.pdf`, max edge 320, 500 iterations | #50 row-kernel boundary | 0.111 ms | 0.085 ms | 23.4% faster |
| `mobile-ocr-overlay-scan.pdf`, max edge 320, 1000 iterations | #51 direct image | 0.087 ms | 0.085 ms | 2.3% faster |
| `scanned-page.pdf`, max edge 320, 1000 iterations | #51 direct image | 0.043 ms | 0.042 ms | 2.3% faster |

The scan path is intentionally reported as a small win. A repeat at max edge
640 for `mobile-ocr-overlay-scan.pdf` moved the other way in this local run
(`0.078 ms` baseline to `0.085 ms` branch), so this PR should not claim a broad
scan speedup beyond the direct-route fixtures above.

## Route Evidence

The #51 direct-image route still classifies the intended fixtures:

| Fixture | `classifier_calls` | `candidate_pages` | `direct_image_calls` | Fallbacks |
| --- | ---: | ---: | ---: | ---: |
| `mobile-ocr-overlay-scan.pdf` | 1 | 1 | 1 | 0 |
| `scanned-page.pdf` | 1 | 1 | 1 | 0 |

The #50 row-kernel change is still below the existing `CoverageDrawBlitter`
selection. It does not add a new renderer routing layer and does not add
architecture-specific intrinsics. Future NEON/AVX/WASM kernels can replace the
safe scalar functions inside `ferrugo-simd` without changing render/native
crate unsafe policy.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p ferrugo-simd --no-default-features -- --nocapture
cargo test -p ferrugo-render fill_edge_intersection_y_break_cache --no-default-features -- --nocapture
cargo test -p ferrugo-render fill_path_coverage_spans_should_use_source_over_row_blitter_for_translucent_normal --no-default-features -- --nocapture
cargo test -p ferrugo-native scanned_page_fast_path --no-default-features -- --nocapture
cargo test -p ferrugo-native direct_scanned_page_thumbnail_should_match_generic_image_raster --no-default-features -- --nocapture
cargo test -p ferrugo-render --no-default-features
cargo test -p ferrugo-native --no-default-features
cargo test -p ferrugo --no-default-features
cargo build --release -p ferrugo --no-default-features
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 300 --max-ms 1000000 --max-output-bytes 1048576 --output target/issues49-50-51-branch3-vector-stress.json
target/release/ferrugo benchmark-native fixtures/generated/blend-modes.pdf --max-edge 320 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/issues49-50-51-branch3-blend-modes.json
target/release/ferrugo benchmark-native fixtures/generated/transparency-alpha.pdf --max-edge 320 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/issues49-50-51-branch3-transparency-alpha.json
target/release/ferrugo benchmark-native fixtures/generated/mobile-ocr-overlay-scan.pdf --max-edge 320 --iterations 1000 --max-ms 1000000 --max-output-bytes 1048576 --output target/issues49-50-51-branch4-mobile-ocr-320.json
target/release/ferrugo benchmark-native fixtures/generated/scanned-page.pdf --max-edge 320 --iterations 1000 --max-ms 1000000 --max-output-bytes 1048576 --output target/issues49-50-51-branch-scanned-page-320.json
target/release/ferrugo trace-native fixtures/generated/mobile-ocr-overlay-scan.pdf --max-edge 320 --output target/issues49-50-51-branch-mobile-ocr-trace.json
target/release/ferrugo trace-native fixtures/generated/scanned-page.pdf --max-edge 320 --output target/issues49-50-51-branch-scanned-page-trace.json
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
```
