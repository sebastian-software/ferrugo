# PDFium Binding Removal 2026-07-04

Issue: #116

## Decision

Ferrugo no longer ships a PDFium binding crate or a `pdfium` Cargo feature.
PDFium remains available only as an external process oracle for maintainer
benchmark and visual comparison workflows.

This supersedes the earlier retain-maintainer-tooling decisions for the
in-process binding. The replacement oracle path landed in #109:

- `benchmark-matrix --backend pdfium`
- `visual-diff`
- `--pdfium PATH`
- `FERRUGO_PDFIUM_RENDERER`

## Removed

- `crates/ferrugo-pdfium`
- `ferrugo` optional dependency on `ferrugo-pdfium`
- `pdfium` Cargo feature
- `render-pdfium`
- `render-isolated`
- private `render-worker`
- `compare-metadata`
- `benchmark-pdfium`
- `ferrugo-pdfium` publish and package-readiness entries

## Retained

- External PDFium renderer configuration for matrix and visual-diff work.
- Historical PDFium reports as archived comparison evidence.
- PDFium checkout documentation for maintainers who build an external renderer
  adapter.

## Gate

`scripts/check_pdfium_quarantine.sh` now fails if the workspace regains the
binding crate, a `ferrugo-pdfium` dependency edge, binding symbols, PDFium
dynamic-library configuration, or packaged native PDFium assets.

## Validation

The implementation PR should record:

- `cargo check -p ferrugo --all-features`
- `cargo test -p ferrugo --all-features`
- `bash scripts/check_pdfium_quarantine.sh`
- `bash scripts/check_plugin_free_distribution.sh`
- `bash scripts/check_crate_publish_ready.sh`
- external PDFium missing-tool matrix and visual-diff smoke checks
