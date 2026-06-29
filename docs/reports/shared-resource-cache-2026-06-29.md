# Shared Resource Cache 2026-06-29

Milestone: 0188

## Summary

The bounded multi-page renderer now loads the PDF document and page tree once
per render request and shares that immutable object graph across scoped page
workers. This removes repeated object-table parsing for long-document preview
batches while preserving the existing `isolated-render` cache policy.

This milestone does not add a global process cache and does not retain decoded
fonts, decoded images, pixels, or decrypted content beyond the render request.
Decoded page resources remain page-local until a document-session cache has
explicit byte accounting, eviction, and tenant lifetime policy.

## Implementation

`render_pages_parallel_partial_with_limits` now:

- loads source bytes once;
- returns early for empty page requests without parsing;
- loads a single `(ClassicDocument, PageTree)` for the whole page set;
- preserves the linearized first-page loader for single page-zero requests;
- uses the full classic loader for multi-page requests so later pages never read
  from a first-page-only object table;
- shares borrowed references to that immutable document state with scoped page
  workers.

## Manifest

Added `fixtures/shared-resource-cache-manifest.tsv`.

| Family | Fixture | Purpose |
| --- | --- | --- |
| `long-document-shared` | `long-document-navigation-deck.pdf` | Repeated font/image resources across a 12-page document. |
| `repeated-font-image` | `longform-repeated-resources.pdf` | Shared font and image resources across longform pages. |
| `repeated-image-xobject` | `image-heavy-repeated-xobject-report.pdf` | Repeated image XObject placement workload. |
| `repeated-font-program` | `subset-type3-repeated-charprocs.pdf` | Repeated Type3 CharProc/font-program workload. |
| `shared-icc` | `icc-rgb-image.pdf` | ICC transform cache policy coverage. |

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/shared-resource-cache-manifest.tsv --include-family long-document-shared --include-family repeated-font-image --include-family repeated-image-xobject --include-family repeated-font-program --include-family shared-icc --fail-on-fallback --max-edge 160 --output target/shared-resource-cache-0188-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Error classes |
| ---: | ---: | ---: | --- |
| 5 | 5 | 0 | `{}` |

## Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/shared-resource-cache-manifest.tsv --include-family long-document-shared --include-family repeated-font-image --include-family repeated-image-xobject --include-family repeated-font-program --include-family shared-icc --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/shared-resource-cache-0188-benchmark.json
```

Result:

| Family | Total | Native | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `long-document-shared` | 1 | 1 | 0 | 0 | 5.173 | 5.173 | 72960 |
| `repeated-font-image` | 1 | 1 | 0 | 0 | 3.982 | 3.982 | 76800 |
| `repeated-font-program` | 1 | 1 | 0 | 0 | 1.183 | 1.183 | 47360 |
| `repeated-image-xobject` | 1 | 1 | 0 | 0 | 13.594 | 13.594 | 85120 |
| `shared-icc` | 1 | 1 | 0 | 0 | 1.820 | 1.820 | 57600 |

## Batch Profile

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/shared-resource-cache-manifest.tsv --include-family long-document-shared --include-family repeated-font-image --include-family repeated-image-xobject --include-family repeated-font-program --include-family shared-icc --repetitions 2 --max-workers 2 --max-in-flight-pixels 51200 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/shared-resource-cache-0188-batch.json
```

Result: 5 inputs, 10 jobs, 10 native rendered, 0 fallbacks, 0 errors, 0 budget
failures, 238.642 jobs/sec. RSS metrics are unavailable on this host; the
recorded memory signal is the enforced `max_in_flight_pixels = 51200`
scheduler budget plus `max_output_bytes = 85120`.

## Follow-Up

The next decoded-resource cache step should be a document-session cache with:

- byte budgets for decoded font programs, CMaps, image samples, and ICC
  transforms;
- hit/miss/eviction counters;
- no disk persistence by default;
- invalidation by document identity and native profile;
- no retention of decrypted or security-sensitive content outside the caller's
  trust boundary.

## Validation

Commands run:

```sh
cargo test -p ferrugo-native parallel_renderer -- --nocapture
cargo test -p ferrugo-native preview_partial -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/shared-resource-cache-manifest.tsv --include-family long-document-shared --include-family repeated-font-image --include-family repeated-image-xobject --include-family repeated-font-program --include-family shared-icc --fail-on-fallback --max-edge 160 --output target/shared-resource-cache-0188-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/shared-resource-cache-manifest.tsv --include-family long-document-shared --include-family repeated-font-image --include-family repeated-image-xobject --include-family repeated-font-program --include-family shared-icc --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/shared-resource-cache-0188-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/shared-resource-cache-manifest.tsv --include-family long-document-shared --include-family repeated-font-image --include-family repeated-image-xobject --include-family repeated-font-program --include-family shared-icc --repetitions 2 --max-workers 2 --max-in-flight-pixels 51200 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/shared-resource-cache-0188-batch.json
```
