# Long Document Navigation Page Cache Gate 2026-06-26

Milestone: 0171

## Summary

Added a focused long-document navigation gate for the Rust-native renderer. The
gate measures first-page, next-page, random-page, repeated render, and bounded
batch behavior without adding a default persistent page cache.

The existing native policy remains `isolated-render`: render passes keep only
pass-local caches, and any longer-lived page artifact cache must be
caller-owned with explicit document identity, render options, native profile,
and eviction policy.

## Fixture Coverage

Added `fixtures/long-document-navigation-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `navigation-longdoc` | 1 | New 12-page fixture for first, next, and random-page sampling. |
| `book-navigation` | 1 | Existing 5-page book fixture with labels and outlines. |
| `repeated-resources` | 1 | Existing repeated font/image longform fixture. |
| `report-sampling` | 1 | Existing 3-page report sampling fixture. |
| `statement-navigation` | 1 | Existing 2-page statement/report navigation baseline. |

New generated fixture:

- `fixtures/generated/long-document-navigation-deck.pdf`

It reuses one image XObject and one font across 12 pages and is included in the
main corpus manifest with `expected:native`.

## Native Coverage

Added:

- `native_parallel_renderer_should_sample_generated_long_document_navigation_pages`
  renders pages 0, 1, 11, and 5 through the bounded parallel scheduler.
- `native_page_cache_key_should_isolate_long_document_navigation_pages` verifies
  page index and background changes produce distinct caller-owned cache keys.

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/long-document-navigation-manifest.tsv \
  --include-family navigation-longdoc \
  --include-family book-navigation \
  --include-family repeated-resources \
  --include-family report-sampling \
  --include-family statement-navigation \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/longdoc-0171-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 5 | 5 | 0 | 0 |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/long-document-navigation-manifest.tsv \
  --include-family navigation-longdoc \
  --include-family book-navigation \
  --include-family repeated-resources \
  --include-family report-sampling \
  --include-family statement-navigation \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/longdoc-0171-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `book-navigation` | 1 | 1 | 0 | 0 | 0 | 22.850 | 22.850 | 74240 |
| `navigation-longdoc` | 1 | 1 | 0 | 0 | 0 | 18.189 | 18.189 | 72960 |
| `repeated-resources` | 1 | 1 | 0 | 0 | 0 | 16.151 | 16.151 | 76800 |
| `report-sampling` | 1 | 1 | 0 | 0 | 0 | 39.663 | 39.663 | 74880 |
| `statement-navigation` | 1 | 1 | 0 | 0 | 0 | 26.462 | 26.462 | 62720 |

## Repeat And Cache Policy Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated \
  --manifest fixtures/long-document-navigation-manifest.tsv \
  --include-family navigation-longdoc \
  --include-family book-navigation \
  --include-family repeated-resources \
  --include-family report-sampling \
  --include-family statement-navigation \
  --repetitions 3 \
  --max-edge 160 \
  --max-first-ms 1000 \
  --max-repeat-mean-ms 1000 \
  --max-errors 0 \
  --fail-on-budget \
  --output target/longdoc-0171-repeat-benchmark.json
```

Result: 5 total, 5 native rendered, 0 fallbacks, 0 errors, 0 budget failures.
The report records `cache_policy.name = isolated-render` and
`permits_disk_persistence = false`.

| Family | First mean ms | Repeat mean ms | Repeat / first |
| --- | ---: | ---: | ---: |
| `book-navigation` | 21.664 | 20.931 | 0.966 |
| `navigation-longdoc` | 18.309 | 17.893 | 0.977 |
| `repeated-resources` | 15.858 | 15.861 | 1.000 |
| `report-sampling` | 39.579 | 39.085 | 0.988 |
| `statement-navigation` | 26.690 | 26.179 | 0.981 |

## Batch Memory Profile

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/long-document-navigation-manifest.tsv \
  --include-family navigation-longdoc \
  --include-family book-navigation \
  --include-family repeated-resources \
  --include-family report-sampling \
  --include-family statement-navigation \
  --repetitions 2 \
  --max-workers 2 \
  --max-in-flight-pixels 51200 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --output target/longdoc-0171-batch-memory.json
```

Result: 10 jobs, 10 native rendered, 0 fallbacks, 0 errors, 0 budget failures,
62.696 jobs/sec, p95 39.743 ms, max output 76800 bytes. RSS metrics are not
available on this host, so the recorded memory signal is the enforced
`max_in_flight_pixels = 51200` scheduler budget plus output-byte ceilings.

## Visual Comparison

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/long-document-navigation-manifest.tsv \
  --include-family navigation-longdoc \
  --include-family book-navigation \
  --include-family repeated-resources \
  --include-family report-sampling \
  --include-family statement-navigation \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/longdoc-0171-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 0 | 0 | 5 | 0 | 0 |

Blockers are existing geometry/text/image fidelity follow-ups, not native
support or navigation failures.

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo test -p pdfrust-native long_document_navigation -- --nocapture`
- `cargo test -p pdfrust-native native_page_cache -- --nocapture`
- Native supported gate, benchmark, repeat benchmark, batch memory profile, and
  visual comparison commands listed above.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
