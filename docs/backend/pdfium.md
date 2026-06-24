# PDFium Backend

Status: Phase 0 local backend shell.
Date: 2026-06-24.

The PDFium backend is isolated in `crates/pdfrust-pdfium`. Public consumers use
the backend-neutral types from `pdfrust-thumbnail`; PDFium handles and symbols
do not appear in the facade API.

## Runtime Configuration

Set `PDFRUST_PDFIUM_LIBRARY` to a local dynamic library built from the pinned
PDFium checkout:

```sh
export PDFRUST_PDFIUM_LIBRARY="../pdfium-work/pdfium/out/pdfrust-thumb/libpdfium.dylib"
```

The exact output filename depends on the PDFium target and platform. Keep the
path local; do not commit PDFium binaries.

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

In this environment the probe was not run because no local PDFium library is
available yet. The code path is covered by unit tests for configuration and by
`cargo check`.
