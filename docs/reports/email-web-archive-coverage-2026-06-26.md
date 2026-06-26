# Email Client And Web Archive Coverage 2026-06-26

Milestone: 0168

## Summary

Added a focused email/web-archive fixture slice covering common print-to-PDF
inputs from mail clients and saved web pages. The slice combines four new
generated PDFs with existing embedded-file, file-attachment, and portfolio
policy fixtures.

The native renderer renders the full slice without fallbacks or errors. Visual
comparison remains useful as fidelity telemetry: the text/image-heavy samples
still show blocker-level pixel drift against PDFium, but no runtime failures.

## Fixture Coverage

Added `fixtures/email-web-archive-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `email-thread` | 1 | Three-page email thread with headers, quoted replies, and repeated panels. |
| `inline-image-link` | 1 | Mobile mail-style inline image plus inert link annotation. |
| `web-archive` | 1 | Saved web archive article/sidebar layout with inert link annotation. |
| `attachment-policy` | 4 | New email attachment summary plus existing embedded-file, file-attachment annotation, and portfolio policy fixtures. |

The four new generated PDFs are also included in the main corpus manifest under
`email-web-archive`.

## Attachment Policy

The renderer keeps embedded payloads inert. `email-attachment-summary.pdf`
exposes `/Names /EmbeddedFiles` metadata, renders the visible summary page, and
does not attempt attachment extraction or execution.

Regression coverage:

- `native_backend_should_inspect_generated_email_attachment_policy_fixture`
  verifies embedded-file presence without portfolio or file-attachment
  annotation flags.
- Existing portfolio and file-attachment fixtures remain in the focused
  manifest as policy baselines.

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/email-web-archive-manifest.tsv \
  --include-family email-thread \
  --include-family inline-image-link \
  --include-family web-archive \
  --include-family attachment-policy \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/email-web-0168-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 7 | 7 | 0 | 0 |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/email-web-archive-manifest.tsv \
  --include-family email-thread \
  --include-family inline-image-link \
  --include-family web-archive \
  --include-family attachment-policy \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/email-web-0168-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `attachment-policy` | 4 | 4 | 0 | 0 | 0 | 19.719 | 26.786 | 235200 |
| `email-thread` | 1 | 1 | 0 | 0 | 0 | 23.977 | 23.977 | 72960 |
| `inline-image-link` | 1 | 1 | 0 | 0 | 0 | 22.436 | 22.436 | 86400 |
| `web-archive` | 1 | 1 | 0 | 0 | 0 | 24.840 | 24.840 | 90880 |

## Low-Memory Thread Check

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/email-web-archive-manifest.tsv \
  --include-family email-thread \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --native-profile low-memory \
  --output target/email-web-0168-low-memory-thread.json
```

Result:

| Total | Native rendered | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 0 | 0 | 0 | 28.067 | 28.067 | 72960 |

`native_parallel_renderer_should_sample_generated_email_thread_pages` also
samples pages 1 and 3 of the three-page thread through the bounded parallel
scheduler.

## Maintainer Visual Comparison

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/email-web-archive-manifest.tsv \
  --include-family email-thread \
  --include-family inline-image-link \
  --include-family web-archive \
  --include-family attachment-policy \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/email-web-0168-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 1 | 2 | 4 | 0 | 0 |

Blockers by subsystem:

| Subsystem | Blockers | Notes |
| --- | ---: | --- |
| `rendering-core` | 3 | Text/layout pixel drift in email thread, web archive, and attachment summary. |
| `images-color` | 1 | Inline-image interpolation/color drift in the mail preview fixture. |

These are fidelity follow-ups, not native support or policy failures.

## Validation

- `cargo test -p pdfrust-native email -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks ...`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native ...`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native ... --native-profile low-memory`
- `cargo run -p pdfrust-cli --features pdfium -- visual-diff ...`
