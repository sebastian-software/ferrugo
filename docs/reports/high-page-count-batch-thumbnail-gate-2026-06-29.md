# High Page Count Batch Thumbnail Gate 2026-06-29

Milestone 0195 makes high-page-count thumbnail generation observable through
the native batch benchmark without committing very large PDF fixtures.

## Implementation

- Added `benchmark-batch-native --pages-per-input N`.
- Batch jobs now fan out by repetition, input path, and page index in stable
  order.
- When a manifest is supplied, page fanout is bounded by each fixture's
  declared page count.
- Batch records continue to report `page_index`, outcome, latency, and family.
- Batch JSON config now records `pages_per_input`.

The default remains one page per input, preserving existing server throughput
gates. High-page-count gates opt in explicitly.

## Fixture Gate

New focused manifest:

- `fixtures/high-page-count-batch-manifest.tsv`

It reuses existing generated fixtures:

| Family | Fixture | Pages |
| --- | --- | --- |
| `long-document` | `long-document-navigation-deck.pdf` | 12 |
| `book` | `book-frontmatter-page-labels.pdf` | 5 |
| `email-thread` | `email-client-thread.pdf` | 3 |
| `repeated-resources` | `longform-repeated-resources.pdf` | 3 |
| `report-statement` | `multi-page-report.pdf` | 2 |

With `--repetitions 10 --pages-per-input 12`, manifest-bounded page fanout
produces 250 page jobs.

## Main Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/high-page-count-batch-manifest.tsv \
  --include-family long-document \
  --include-family book \
  --include-family email-thread \
  --include-family repeated-resources \
  --include-family report-statement \
  --repetitions 10 \
  --pages-per-input 12 \
  --max-workers 4 \
  --max-in-flight-pixels 102400 \
  --max-edge 160 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --fail-on-budget \
  --output target/high-page-count-0195-batch.json
```

Result:

| Total inputs | Total jobs | Native rendered | Fallbacks | Errors | Budget failures |
| --- | --- | --- | --- | --- | --- |
| 5 | 250 | 250 | 0 | 0 | 0 |

Performance and bounded memory signals:

| Throughput/sec | Mean ms | P50 ms | P95 ms | Max ms | Max in-flight pixels | Max output bytes |
| --- | --- | --- | --- | --- | --- | --- |
| 360.289 | 7.079 | 5.291 | 25.418 | 28.953 | 102400 | 76800 |

RSS fields were unavailable on this macOS run and are recorded as `null` in the
JSON artifact.

## Cancellation Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/high-page-count-batch-manifest.tsv \
  --include-family long-document \
  --include-family book \
  --include-family email-thread \
  --include-family repeated-resources \
  --include-family report-statement \
  --repetitions 10 \
  --pages-per-input 12 \
  --max-workers 4 \
  --max-in-flight-pixels 102400 \
  --max-edge 160 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --cancel-after-jobs 25 \
  --output target/high-page-count-0195-cancelled.json
```

Result:

| Scheduled jobs | Skipped jobs | Cancelled | Native rendered | Fallbacks | Errors |
| --- | --- | --- | --- | --- | --- |
| 25 | 225 | true | 25 | 0 | 0 |

The cancellation boundary stops queued work while preserving completed page
results and deterministic ordering.

## Low-Memory Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/high-page-count-batch-manifest.tsv \
  --include-family long-document \
  --include-family book \
  --include-family email-thread \
  --include-family repeated-resources \
  --include-family report-statement \
  --repetitions 3 \
  --pages-per-input 12 \
  --max-workers 2 \
  --max-in-flight-pixels 51200 \
  --max-edge 160 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --fail-on-budget \
  --native-profile low-memory \
  --output target/high-page-count-0195-low-memory.json
```

Result:

| Total inputs | Total jobs | Native rendered | Fallbacks | Errors | Budget failures |
| --- | --- | --- | --- | --- | --- |
| 5 | 75 | 75 | 0 | 0 | 0 |

| Throughput/sec | Mean ms | P50 ms | P95 ms | Max ms | Max in-flight pixels | Max output bytes |
| --- | --- | --- | --- | --- | --- | --- |
| 221.785 | 6.964 | 5.185 | 25.334 | 28.954 | 51200 | 76800 |

## Tests

Targeted tests:

```sh
cargo test -p pdfrust-cli batch -- --nocapture
```

Results:

- 4 passed.
- Coverage includes config parsing, worker budget calculation, page fanout
  ordering, throughput/error reporting, and cooperative cancellation.
