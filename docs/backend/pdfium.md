# PDFium Backend

Status: Phase 0 local backend shell.
Date: 2026-06-24.

The PDFium backend is isolated in `crates/pdfrust-pdfium`. Public consumers use
the backend-neutral types from `pdfrust-thumbnail`; PDFium handles and symbols
do not appear in the facade API.

PDFium support is optional for `pdfrust-cli`. Default CLI builds are
native-only. Use `--features pdfium` when running PDFium commands, fallback
renders, metadata comparisons, or PDFium benchmarks.

## Runtime Configuration

Set `PDFRUST_PDFIUM_LIBRARY` to a local dynamic library built from the pinned
PDFium checkout:

```sh
export PDFRUST_PDFIUM_LIBRARY="/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib"
export DYLD_LIBRARY_PATH="/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib"
```

The component dylib depends on other `@rpath` dylibs in the same output
directory, so `DYLD_LIBRARY_PATH` is required for this local probe. Keep these
paths local; do not commit PDFium binaries.

## Serialization

PDFium calls are serialized through a process-local mutex in the backend crate.
Phase 0 deliberately favors conservative correctness over concurrent throughput.
Worker pools and process isolation remain later decisions.

## Smoke Probe

The backend shell can load a local PDFium dynamic library, call
`FPDF_InitLibrary`, read `FPDF_GetLastError`, and call `FPDF_DestroyLibrary`.
This validates runtime linkage without exposing PDFium state through the public
thumbnail API.

## RGBA Rendering

The render path opens borrowed bytes or file input, loads the requested page,
scales it so neither dimension exceeds `max_edge`, renders through a PDFium
bitmap, and converts PDFium BGRA rows into the facade's RGBA buffer. File reads
happen before entering the serialized PDFium section so unrelated I/O does not
hold the global backend lock.

In this environment the probe completed against the pinned local PDFium build:

```sh
cargo run -p pdfrust-pdfium --example smoke
```

```text
initialized=true last_error=0 library=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib
```

## CLI Example

After `PDFRUST_PDFIUM_LIBRARY` points at a local PDFium build:

```sh
cargo run -p pdfrust-cli --features pdfium -- render-pdfium fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

For product-facing timeout behavior, use the isolated runner. It spawns a
single-use worker process and kills it if the wall-clock timeout expires:

```sh
cargo run -p pdfrust-cli --features pdfium -- render-isolated fixtures/generated/text-page.pdf \
  --output target/pdfrust-thumbnails/text-page.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

`render-worker` is a private child-process entry point. Callers should use
`render-pdfium` for direct trusted PDFium probes or `render-isolated` for hard
timeout semantics. The public `render` command is native-first automatic mode.
