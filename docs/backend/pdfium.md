# PDFium Backend

Status: maintainer-only optional oracle backend.
Date: 2026-06-29.

The PDFium backend is isolated in `crates/ferrugo-pdfium`. Public consumers use
the backend-neutral types from `ferrugo-thumbnail`; PDFium handles and symbols
do not appear in the facade API.

PDFium support is optional for `ferrugo-cli`. Default CLI builds are
native-only. Use `--features pdfium` only when running explicit maintainer
commands such as PDFium renders, metadata comparisons, PDFium benchmarks, or
visual diffs. Native-default `render` / `render-auto` commands do not retry
through PDFium.

Milestone 0215 retains this backend as quarantined comparison tooling. It is not
a supported runtime dependency or a release prerequisite.

Consumers migrating away from PDFium should depend on `ferrugo-thumbnail` plus
`ferrugo-native`, branch on `ThumbnailError::class()`, and treat `unsupported`
as the stable native outcome for documents outside the current renderer
boundary. The full policy lives in
`docs/policies/native-renderer-api-semver.md`.

## Runtime Configuration

Set `FERRUGO_PDFIUM_LIBRARY` to a local dynamic library built from the pinned
PDFium checkout:

```sh
export FERRUGO_PDFIUM_LIBRARY="/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib"
export DYLD_LIBRARY_PATH="/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib"
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
cargo run -p ferrugo-pdfium --example smoke
```

```text
initialized=true last_error=0 library=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib
```

## CLI Example

After `FERRUGO_PDFIUM_LIBRARY` points at a local PDFium build:

```sh
cargo run -p ferrugo-cli --features pdfium -- render-pdfium fixtures/generated/text-page.pdf \
  --output target/ferrugo-thumbnails/text-page.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

For product-facing timeout behavior, use the isolated runner. It spawns a
single-use worker process and kills it if the wall-clock timeout expires:

```sh
cargo run -p ferrugo-cli --features pdfium -- render-isolated fixtures/generated/text-page.pdf \
  --output target/ferrugo-thumbnails/text-page.png \
  --page-index 0 \
  --max-edge 1024 \
  --background '#ffffff' \
  --timeout 5
```

`render-worker` is a private child-process entry point. Callers should use
`render-pdfium` for direct trusted PDFium probes or `render-isolated` for hard
timeout semantics. The public `render` command is native-only runtime rendering.
Direct `render-worker` invocation is guarded by the internal
`FERRUGO_PDFIUM_RENDER_WORKER` marker, which `render-isolated` sets only for its
child process.

## Quarantine Check

Run the PDFium quarantine check before changing feature flags, CLI dispatch, or
runtime crate dependencies:

```sh
bash scripts/check_pdfium_quarantine.sh
```

The check confirms that native-only `ferrugo-cli` has no `ferrugo-pdfium`
dependency edge and that runtime crates do not contain forbidden PDFium
integration symbols.
