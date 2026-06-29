# Incremental Streaming Memory Budget 2026-06-29

Milestone: 0187

## Summary

The native backend now exposes first-page preview loader memory metrics through
`FirstPagePreviewMemory`. The preview API reports the selected load mode plus
the input byte length, parsed object count, parsed object byte span, declared
linearized first-page section size, and whether the selected loader stayed
inside that first-page section.

This is a bounded local-input improvement, not a remote range-fetching
implementation. Valid linearized page-zero previews use the first-page object
loader. Malformed linearization hints, non-linearized inputs, unsupported
linearized structures, remote transports, and non-zero page preview requests
still require full-file availability for correctness.

## API And Loader Boundary

| API | Added signal |
| --- | --- |
| `FirstPagePreview::memory` | Stable memory-relevant loader metrics. |
| `FirstPagePreviewMemory::loaded_objects` | Parsed indirect object count retained by the selected loader. |
| `FirstPagePreviewMemory::loaded_object_bytes` | Sum of parsed indirect-object byte spans retained by the selected loader. |
| `FirstPagePreviewMemory::first_page_only` | True when the linearized first-page section was sufficient. |

The preview path was refactored so it renders the same loaded document instance
used for load-mode and memory reporting. It no longer needs a second parser
pass just to render the page-zero thumbnail.

Focused tests assert that `linearized-first-page.pdf` retains fewer parsed
objects and parsed object bytes than the full classic loader for the same
fixture. The malformed linearization fixture reports `full-document` and
`first_page_only = false` while still rendering page zero.

## Manifest

Added `fixtures/incremental-memory-budget-manifest.tsv`.

| Family | Fixture | Purpose |
| --- | --- | --- |
| `linearized-first-page` | `linearized-first-page.pdf` | Bounded first-page object retention. |
| `full-loader-fallback` | `linearized-malformed-hints.pdf` | Correct full-document fallback for bad linearization. |
| `long-document` | `long-document-navigation-deck.pdf` | Long document page preview memory profile. |
| `repeated-resources` | `longform-repeated-resources.pdf` | Reused font/image resource retention signal. |
| `page-targeted` | `page-targeted-stream.pdf` | Page zero avoids decoding a malformed secondary stream. |
| `large-resource` | `scanner-large-image-budget.pdf` | Large image resource decode budget coverage. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/incremental-memory-budget-manifest.tsv --include-family linearized-first-page --include-family full-loader-fallback --include-family long-document --include-family repeated-resources --include-family page-targeted --include-family large-resource --fail-on-fallback --max-edge 160 --output target/incremental-memory-0187-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Error classes |
| ---: | ---: | ---: | --- |
| 6 | 6 | 0 | `{}` |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/incremental-memory-budget-manifest.tsv --include-family linearized-first-page --include-family full-loader-fallback --include-family long-document --include-family repeated-resources --include-family page-targeted --include-family large-resource --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/incremental-memory-0187-benchmark.json
```

Result:

| Family | Total | Native | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `full-loader-fallback` | 1 | 1 | 0 | 0 | 4.803 | 4.803 | 57600 |
| `large-resource` | 1 | 1 | 0 | 0 | 13.527 | 13.527 | 74240 |
| `linearized-first-page` | 1 | 1 | 0 | 0 | 5.352 | 5.352 | 57600 |
| `long-document` | 1 | 1 | 0 | 0 | 5.414 | 5.414 | 72960 |
| `page-targeted` | 1 | 1 | 0 | 0 | 1.152 | 1.152 | 38400 |
| `repeated-resources` | 1 | 1 | 0 | 0 | 4.063 | 4.063 | 76800 |

## Batch Memory Profile

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/incremental-memory-budget-manifest.tsv --include-family linearized-first-page --include-family full-loader-fallback --include-family long-document --include-family repeated-resources --include-family page-targeted --include-family large-resource --repetitions 2 --max-workers 2 --max-in-flight-pixels 51200 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/incremental-memory-0187-batch-memory.json
```

Result: 6 inputs, 12 jobs, 12 native rendered, 0 fallbacks, 0 errors, 0 budget
failures, 223.004 jobs/sec. RSS metrics are unavailable on this host, so the
recorded memory signal is the enforced `max_in_flight_pixels = 51200`
scheduler budget plus `max_output_bytes = 76800`.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p pdfrust-native first_page_preview -- --nocapture
cargo test -p pdfrust-object linearized -- --nocapture
cargo test -p pdfrust-native long_document -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/incremental-memory-budget-manifest.tsv --include-family linearized-first-page --include-family full-loader-fallback --include-family long-document --include-family repeated-resources --include-family page-targeted --include-family large-resource --fail-on-fallback --max-edge 160 --output target/incremental-memory-0187-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/incremental-memory-budget-manifest.tsv --include-family linearized-first-page --include-family full-loader-fallback --include-family long-document --include-family repeated-resources --include-family page-targeted --include-family large-resource --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/incremental-memory-0187-benchmark.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/incremental-memory-budget-manifest.tsv --include-family linearized-first-page --include-family full-loader-fallback --include-family long-document --include-family repeated-resources --include-family page-targeted --include-family large-resource --repetitions 2 --max-workers 2 --max-in-flight-pixels 51200 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/incremental-memory-0187-batch-memory.json
```
