# Incremental Preview Boundary

Date: 2026-06-26.
Milestone: 0155.

## Summary

The native backend now exposes an explicit incremental preview boundary without
introducing PDFium or viewer UI behavior.

New API surface:

| API | Purpose |
| --- | --- |
| `NativeBackend::render_first_page_preview` | Renders page zero and reports whether the linearized first-page loader was usable. |
| `FirstPagePreviewLoadMode` | Stable load-mode enum: `linearized-first-page` or `full-document`. |
| `NativeBackend::render_preview_pages_partial` | Renders requested preview pages with page-level outcomes, cooperative cancellation, and backend-specific render limits. |

The previous `render_pages_parallel_partial` free function remains available
and keeps its existing default-limit behavior.

## Preview Manifest

The focused preview manifest is
`fixtures/incremental-preview-manifest.tsv`.

| Family | Fixture | Coverage |
| --- | --- | --- |
| `first-page-linearized` | `linearized-first-page.pdf` | Valid linearized-style first-page section. |
| `first-page-fallback` | `linearized-malformed-hints.pdf` | Malformed linearization hints falling back to full-document loading. |
| `page-targeted` | `page-targeted-stream.pdf` | Page zero renders without decoding a malformed page-one stream. |
| `multipage-preview` | `multi-page-report.pdf` | Partial preview and cancellation scheduling baseline. |
| `longform-preview` | `longform-repeated-resources.pdf` | Longform repeated-resource preview benchmark. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/incremental-preview-manifest.tsv --include-family first-page-linearized --include-family first-page-fallback --include-family page-targeted --include-family multipage-preview --include-family longform-preview --fail-on-fallback --max-edge 160 --output target/preview-0155-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 5 | 5 | 0 | 0 |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/incremental-preview-manifest.tsv --include-family first-page-linearized --include-family first-page-fallback --include-family page-targeted --include-family multipage-preview --include-family longform-preview --max-edge 160 --iterations 3 --max-ms 1000 --max-output-bytes 1048576 --output target/preview-0155-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `first-page-fallback` | 1 | 25.780 | 25.780 | 0 |
| `first-page-linearized` | 1 | 26.098 | 26.098 | 0 |
| `longform-preview` | 1 | 17.101 | 17.101 | 0 |
| `multipage-preview` | 1 | 26.661 | 26.661 | 0 |
| `page-targeted` | 1 | 5.212 | 5.212 | 0 |

Low-memory longform command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/incremental-preview-manifest.tsv --include-family longform-preview --max-edge 160 --iterations 3 --max-ms 1000 --max-output-bytes 1048576 --native-profile low-memory --output target/preview-0155-low-memory-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `longform-preview` | 1 | 16.888 | 16.888 | 0 |

## Cancellation And Partial Results

Focused tests cover:

- `render_first_page_preview` reports `LinearizedFirstPage` for
  `linearized-first-page.pdf`.
- Malformed linearization reports `FullDocument` while still rendering page
  zero.
- `render_preview_pages_partial` preserves mixed page outcomes for
  `page-targeted-stream.pdf`: page zero succeeds, page one remains malformed.
- Pre-scheduling cancellation returns an empty cancelled partial result.

## Boundary

- First-page preview is page-zero only and intentionally forces
  `page_index = 0`.
- The current API uses local bytes or local files through `PdfSource`; remote
  range fetching is still out of scope.
- Partial preview preserves deterministic output and page-level errors instead
  of hiding malformed requested pages.
- Cancellation is cooperative and checked before scheduling further page
  batches; already-started page work is allowed to finish.

## Validation

Commands run:

```sh
cargo test -p pdfrust-native first_page_preview -- --nocapture
cargo test -p pdfrust-native preview_partial -- --nocapture
cargo test -p pdfrust-native linearized -- --nocapture
cargo test -p pdfrust-native partial_renderer -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/incremental-preview-manifest.tsv --include-family first-page-linearized --include-family first-page-fallback --include-family page-targeted --include-family multipage-preview --include-family longform-preview --fail-on-fallback --max-edge 160 --output target/preview-0155-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/incremental-preview-manifest.tsv --include-family first-page-linearized --include-family first-page-fallback --include-family page-targeted --include-family multipage-preview --include-family longform-preview --max-edge 160 --iterations 3 --max-ms 1000 --max-output-bytes 1048576 --output target/preview-0155-benchmark.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/incremental-preview-manifest.tsv --include-family longform-preview --max-edge 160 --iterations 3 --max-ms 1000 --max-output-bytes 1048576 --native-profile low-memory --output target/preview-0155-low-memory-benchmark.json
```
