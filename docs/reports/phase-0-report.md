# Phase 0 Report

Status: completed with external PDFium build pending.
Date: 2026-06-24.

Phase 0 created the measurement spine for PDF thumbnail generation: pinned
PDFium inputs, generated fixtures, a backend-neutral Rust API, a serialized
PDFium backend shell, a PNG CLI, error taxonomy, and baseline metadata.

## Summary

The Rust side is operational and validated locally. The PDFium source-build side
is specified but not yet measured because this environment does not have
`depot_tools`, `gclient`, `gn`, `ninja`, or a local PDFium dynamic library.

## Completed Artifacts

- License and attribution policy: `LICENSE-MIT`, `LICENSE-APACHE`,
  `docs/policies/attribution.md`.
- PDFium checkout and build inputs:
  - `docs/build/pdfium-checkout.md`
  - `docs/build/pdfium-gn-args.md`
  - PDFium revision `573758fe2dd928279cd52b5a4bc955a6938aab39`
- Measurement protocol: `docs/measurements/pdfium-build-baseline.md`.
- Fixture policy and seed PDFs:
  - `docs/fixtures.md`
  - `fixtures/generated/*.pdf`
  - `scripts/generate_fixtures.py`
- Rust workspace:
  - `crates/pdfrust-thumbnail`
  - `crates/pdfrust-pdfium`
  - `crates/pdfrust-cli`
- Error taxonomy: `docs/errors.md`.
- Baseline metadata format: `docs/baselines.md`,
  `baselines/examples/*.json`.

## Measurements

No PDFium binary size, startup, render-time, or memory measurements are
available yet. The measurement report records commands and environment context,
but the local build is blocked on toolchain setup.

Available local validation:

- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Fixture regeneration through `python3 scripts/generate_fixtures.py`
- JSON validation for baseline examples

## Error Behavior

The facade exposes stable classes:

- `encrypted`
- `malformed`
- `unsupported`
- `timeout`
- `internal`

PDFium error-code mapping is implemented for known `FPDF_GetLastError` codes.
Timeout remains a facade class and CLI option; enforcement around a live backend
needs the PDFium runtime probe.

## Risks And Blockers

- PDFium build plausibility is still unknown until `gn gen` and `ninja` run.
- The dynamic-library filename and exported symbols must be confirmed against
  the actual cut-down build.
- RGBA render dimensions and PNG output are compiled and unit-tested, but not
  live-validated against generated fixtures.
- Timeout behavior needs either backend cancellation, worker isolation, or
  process isolation before it is robust enough for hostile PDFs.
- The baseline success example uses placeholder pixel digest and dimensions
  until the first local render.

## Decision

Continue with both tracks, but gate product conclusions on live PDFium
measurements:

1. Finish the local PDFium checkout/build and record real size/startup/render
   data.
2. Use the current Rust facade, fixtures, CLI, and baseline format as the test
   spine.
3. Do not start Node-API or npm packaging until the PDFium backend either
   renders fixtures reliably or fails with a documented build/runtime reason.
4. Keep Rust-native renderer work as the long-term direction, but use the
   PDFium-backed baseline to define expected thumbnail behavior first.
