# Font And Glyph Cache Session Batch Report - 2026-07-04

## Scope

Issue #111 targets two text cache gaps:

- repeated parsed-face work during cache-backed glyph outline extraction;
- repeated glyph bitmap work across multi-page native batch renders.

This change keeps the implementation safe under `#![forbid(unsafe_code)]`:

- `GlyphOutlineCache` now primes TrueType outlines for a font program on the first cache-backed lookup. The first miss parses the face once, fills the bounded outline cache for the program up to `max_cache_entries`, and subsequent covered glyph lookups reuse cached outlines without reparsing the face.
- `benchmark-batch-native` now groups contiguous pages from the same input and repetition into a `NativeDocumentSession`. That activates the existing bounded session glyph bitmap cache, font resource cache, and observable session stats for multi-page batch runs.
- Batch JSON records now include per-record native session stats when a document session is used.

CFF and Type 1 outline extraction still use their existing direct extraction path. The visible text renderer remains on the current fallback/Type 3 raster path; this report measures the session glyph bitmap cache that is active in production native renders.

## Benchmark Command

Baseline was run from `main` at `0570e01`. The after run was run from `codex/issue-111-font-glyph-caches`.

```sh
cargo run -p ferrugo --release --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/high-page-count-batch-manifest.tsv \
  --include-family long-document \
  --repetitions 3 \
  --pages-per-input 12 \
  --max-workers 2 \
  --max-in-flight-pixels 28800 \
  --max-edge 120 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --fail-on-budget
```

Fixture: `fixtures/generated/long-document-navigation-deck.pdf`

## Results

| Run | Isolation | Jobs | Native | Errors | Throughput/sec | Mean ms | P50 ms | P95 ms | Max ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Before | per-job isolated render | 36 | 36 | 0 | 2184.571 | 0.257 | 0.184 | 1.223 | 1.247 |
| After | per-input document session | 36 | 36 | 0 | 7993.043 | 0.161 | 0.110 | 0.984 | 0.989 |

The after run used `cache_policy=document-session`, `backend_scope=per-input-session`, and `shared_document_state=true`.

## Session Cache Evidence

At the last page of each 12-page repetition, the retained session reported:

| Counter | Value |
| --- | ---: |
| `cached_glyph_bitmap_hits` | 267 |
| `cached_glyph_bitmap_misses` | 36 |
| `cached_glyph_bitmap_entries` | 36 |
| `cached_glyph_bitmap_bytes` | 16160 |
| `max_cached_glyph_bitmap_entries` | 1024 |
| `max_cached_glyph_bitmap_bytes` | 1048576 |
| `cached_font_resource_hits` | 11 |
| `cached_font_resource_misses` | 1 |

No glyph bitmap evictions were observed in this fixture, and the resident glyph bitmap cache stayed below the configured byte budget.

## Validation

```sh
cargo fmt --check
cargo test -p ferrugo-render glyph_outline_cache_should -- --nocapture
cargo test -p ferrugo-render
cargo test -p ferrugo batch_job_groups_should_group_contiguous_pages_by_input_and_repetition -- --nocapture
cargo test -p ferrugo batch_benchmark_should_report_cooperative_cancellation_boundary -- --nocapture
cargo test -p ferrugo batch_benchmark -- --nocapture
cargo test -p ferrugo
cargo check -p ferrugo --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
```

All commands passed locally.
