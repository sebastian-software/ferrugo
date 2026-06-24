# Rust-Native Backend

Status: supported for generated fixture coverage.
Date: 2026-06-24.

The Rust-native backend lives in `crates/pdfrust-native` and implements the same
`pdfrust-thumbnail` facade traits as the PDFium backend:

- `ThumbnailBackend` for single-page RGBA thumbnail rendering.
- `DocumentMetadataBackend` for page-count and page-size inspection.

Callers can switch between `PdfiumBackend` and `NativeBackend` without changing
the facade input or output types. The native backend returns raw RGBA thumbnail
buffers through `Thumbnail::rgba`; CLI PNG encoding remains owned by
`pdfrust-cli`, matching the PDFium backend path.

## Supported Contract

- `page_index` selects the zero-based page or returns `unsupported` when the
  page is unavailable.
- `max_edge` uses the same scale-down policy as PDFium: the largest page edge is
  clamped to the requested maximum, preserving aspect ratio.
- `background` initializes the page raster before paths, images, text, and
  appearances are painted.
- Encrypted documents return the public `encrypted` class.
- Malformed parser/object structures return the public `malformed` class.
- Unsupported native features and budget exhaustion return the public
  `unsupported` class.

## Fallback Policy

PDFium remains the oracle and broad fallback until the retirement gate says the
native backend covers enough typical documents. Product code should use native
only where the support matrix marks the document class as rendered, or should
retry through PDFium when native returns `unsupported`.

Do not retry native `encrypted` or `malformed` errors through a silent repair
path. Encrypted documents need explicit password/security policy, and malformed
documents should stay diagnosable.

## Local Checks

Use native directly:

```sh
cargo run -p pdfrust-cli -- render-native fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page-native.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

Use native-first automatic rendering for supported categories:

```sh
cargo run -p pdfrust-cli -- render fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page-auto.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

`render` and `render-auto` try the Rust-native backend first. In a
PDFium-enabled CLI build, if native returns the public `unsupported` class, the
command retries through PDFium using `PDFRUST_PDFIUM_LIBRARY`. In the default
native-only CLI build, the same case fails with a usage-oriented diagnostic that
asks for `--features pdfium`. `encrypted`, `malformed`, and `internal` failures
are not silently retried. The selected backend is printed as a render
diagnostic. Fallback diagnostics include `fallback_reason=<bucket>` and
`fallback_category=<bucket>` so corpus runs can count the remaining PDFium
surface.

Use `--native-only` or `--no-pdfium-fallback` with `render`/`render-auto` to
fail CI or release validation when native cannot render a fixture without
PDFium. Use `--deny-fallback-reason <bucket>` for targeted experiments, or set
`PDFRUST_NATIVE_ONLY=1` / `PDFRUST_DENY_FALLBACK_REASONS=bucket.one,bucket.two`
for environment-driven runs.

Use `render-native` to force native without fallback. Use `render-pdfium` or
`render-isolated` to force PDFium in a CLI build compiled with
`--features pdfium`.

Summarize a local corpus without rendering PDFium output:

```sh
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/pdfrust-thumbnails/fallback-summary.json
```

The summary counts `native_rendered`, `fallback_required`,
`fallback_categories`, non-fallback `errors`, and per-family pass rates when a
manifest is provided. Add `--fail-on-fallback` for CI subsets that must stay
native-only.

Extract committed fixture metadata with page sizes and manifest tags:

```sh
cargo run -p pdfrust-cli -- extract-corpus-metadata fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --output target/pdfrust-thumbnails/corpus-metadata.json
```

Compare metadata with PDFium when the local PDFium environment is available:

```sh
cargo run -p pdfrust-cli --features pdfium -- compare-metadata fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page-metadata-comparison.json
```

The comparison JSON includes `rust_native_memory`, which records the default
native memory budget snapshot used for the local run.
