# PDFium External Oracle

Status: external maintainer oracle only.
Date: 2026-07-04.

Ferrugo no longer ships a PDFium binding crate or a `pdfium` Cargo feature.
The native product path is Rust-only. PDFium may still be used as an external
process oracle for local benchmark and visual comparison work.

## Supported Commands

Use the external PDFium renderer through:

- `benchmark-matrix --backend pdfium`
- `visual-diff`

Configure the renderer with `--pdfium PATH` or `FERRUGO_PDFIUM_RENDERER`.
When neither is set, the CLI looks for `pdfium_test` on `PATH`.

The renderer command must accept:

```sh
pdfium_test \
  --input fixtures/generated/text-page.pdf \
  --page-index 0 \
  --max-edge 160 \
  --background '#FFFFFFFF' \
  --output target/pdfium-page.ppm
```

It must write a binary PPM (`P6`) RGB image to the output path. Keep local
PDFium checkout and wrapper binaries outside this repository.

## Removed Binding Surface

These legacy binding commands are intentionally gone:

- `render-pdfium`
- `render-isolated`
- `render-worker`
- `compare-metadata`
- `benchmark-pdfium`

Metadata parity work should use native metadata extraction and reviewed
baseline records until a future external metadata oracle is introduced.

## Quarantine Check

Run the PDFium quarantine check before changing CLI dispatch, package metadata,
or runtime dependencies:

```sh
bash scripts/check_pdfium_quarantine.sh
```

The check fails if the workspace regains `crates/ferrugo-pdfium`, a
`ferrugo-pdfium` dependency edge, PDFium binding symbols, PDFium dynamic-library
configuration, or packaged native PDFium assets.
