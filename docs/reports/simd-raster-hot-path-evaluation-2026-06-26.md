# SIMD Raster Hot Path Evaluation 2026-06-26

Milestone: 0159.

## Decision

Do not retain a SIMD or platform-accelerated raster path in this milestone.
The current scalar renderer remains the only production path.

The project declares `rust-version = "1.81"`. The local toolchain for this run
was `rustc 1.95.0-nightly`, but normal builds must not require nightly-only
portable SIMD APIs. Platform-specific acceleration would also require explicit
feature detection, duplicate scalar/accelerated tests, and long-term parity
maintenance. The benchmark evidence below does not justify that complexity yet.

## Evaluated Hot Paths

| Area | Current scalar state | SIMD decision |
| --- | --- | --- |
| Source-over alpha and blend | Small per-pixel function, already covered by blend tests. | Keep scalar until a wider row-level compositor exists. |
| Path fill and stroke | Dominated by geometry predicates, clip checks, and supersample coverage. | SIMD is not the right first lever; previous bounds pruning was higher impact. |
| Image sampling and conversion | Samples only target pixels and uses `ImageSampleCache` for repeated source samples. | Keep scalar until conversion work dominates measured release profiles. |
| PNG predictor rows | Runs in-place on decoded buffers. | Keep scalar; no retained prototype without row-level benchmarks. |
| Transparency groups | Bounded intermediate raster plus existing scalar compositor. | Keep scalar; visual parity work is more important than dual-path acceleration. |

## Benchmark Evidence

Low-memory representative artifact:
`target/simd-0159-scalar-low-memory-benchmark.json`.

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `common` | 2 | 2 | 15.981 | 31.443 | 0 |
| `repeated-resources` | 1 | 1 | 16.638 | 16.638 | 0 |
| `scan` | 1 | 1 | 42.388 | 42.388 | 0 |
| `vector-stress` | 1 | 1 | 191.063 | 191.063 | 0 |

Transparency/blend artifact:
`target/simd-0159-scalar-transparency-benchmark.json`.

| Family | Total | Native rendered | Fallback required | Mean ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `alpha` | 1 | 1 | 0 | 32.142 | 0 |
| `blend` | 2 | 2 | 0 | 28.022 | 0 |
| `group` | 3 | 3 | 0 | 19.637 | 0 |
| `image-soft-mask` | 1 | 1 | 0 | 1.517 | 0 |
| `unsupported-blend` | 1 | 0 | 1 | 0.000 | 1 |
| `unsupported-soft-mask` | 1 | 0 | 1 | 0.000 | 1 |

The two budget failures are expected typed native unsupported outcomes in the
`graphics.transparency` bucket, not allocator or runtime failures.

## Visual Subset

Visual oracle artifact: `target/simd-0159-transparency-visual-diff.json`.

| Result | Count |
| --- | ---: |
| Exact | 4 |
| Accepted drift | 2 |
| Blockers | 1 |
| Native errors | 0 |
| PDFium errors | 0 |

The remaining blocker is the known `transparency-alpha.pdf` parity issue. SIMD
does not address that semantic mismatch.

## Rejection Notes

- No retained SIMD patch: no meaningful speedup was demonstrated against a
  maintained scalar baseline.
- No nightly dependency: normal builds must remain compatible with the declared
  MSRV.
- No platform-specific dispatch yet: explicit CPU feature detection and parity
  tests should wait for a measured row-level compositor or color-conversion
  prototype.
- Next useful step: add a release-mode microbenchmark harness for source-over,
  image color conversion, and predictor rows before introducing acceleration
  code.

## Validation Commands

```text
rustc --version --verbose
cargo test -p pdfrust-render source_over -- --nocapture
cargo test -p pdfrust-render blend -- --nocapture
cargo test -p pdfrust-render image_sample_cache -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/low-memory-profile-manifest.tsv --include-family common --include-family scan --include-family vector-stress --include-family repeated-resources --max-edge 160 --iterations 3 --max-ms 1000 --max-output-bytes 1048576 --output target/simd-0159-scalar-low-memory-benchmark.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/transparency-conformance-manifest.tsv --include-family alpha --include-family group --include-family blend --include-family image-soft-mask --include-family unsupported-soft-mask --include-family unsupported-blend --max-edge 160 --iterations 3 --max-ms 1000 --max-output-bytes 1048576 --output target/simd-0159-scalar-transparency-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/transparency-conformance-manifest.tsv --include-family alpha --include-family group --include-family blend --include-family image-soft-mask --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/simd-0159-transparency-visual-diff.json
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.
